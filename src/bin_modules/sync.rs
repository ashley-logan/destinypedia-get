use super::Result;
use super::database::rows::{CategoriesRow, ImageCategoryRow, ImagesRow, Row, SubCategoryRow};
use async_channel;
use better_tracing::fmt::{FormatEvent, Subscriber};
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, bounded, unbounded};
use destinypedia::NAMESPACE;
use destinypedia::request::{PARAMS, Query};
use destinypedia::response::{Continue, QueryResponse};
use reqwest::{Client, Response};
use serde_json::from_slice;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{
    Sqlite,
    prelude::*,
    query_builder::{QueryBuilder, Separated},
};
use sqlx::{SqlitePool, query, sqlite::SqlitePoolOptions};
use std::collections::HashMap;
use std::fs;
use std::path;
use std::sync::Arc;
use tokio::task;
use tracing::{self, Level, debug_span};

const USER_AGENT: &'static str = "DESTINY_FETCH";
const BASE: &'static str = "https://www.destinypedia.com/api.php";
const CATEGORY_IMAGES_ID: i32 = 364;
static DEV_DB_URL: &str = "data/dev.db";
const BIND_LIMIT: u16 = 32766;

#[tracing::instrument]
pub fn create_backup<T: AsRef<path::Path> + std::fmt::Debug>(original: T) -> Result<path::PathBuf> {
    let backup: path::PathBuf = original.as_ref().with_added_extension("bak");
    fs::rename(original, backup.as_path())?;
    Ok(backup)
}

#[tracing::instrument]
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    sqlx::migrate!().run(pool).await?;
    Ok(())
}

pub async fn sync(db_url: &str, starting_pageid: Option<i32>) -> Result<()> {
    use tokio::task::Id;
    let span = tracing::debug_span!("attempting to sync the database");
    let _guard = span.enter();
    let start_t = tokio::time::Instant::now();

    let opts = SqliteConnectOptions::new()
        .foreign_keys(false)
        .create_if_missing(true)
        .filename(db_url);
    let pool = SqlitePoolOptions::new().connect_lazy_with(opts);
    tracing::debug!("lazy connection pool created");

    run_migrations(&pool).await?;

    let (pageid_send, pageid_recv) = async_channel::unbounded::<i32>();
    let (id_bytes_send, id_bytes_recv) = unbounded::<(i32, Vec<u8>)>();
    let (row_send, row_recv) = async_channel::unbounded::<Row>();

    let first_id: i32 = starting_pageid.unwrap_or(CATEGORY_IMAGES_ID);

    tracing::debug!(
        first_id,
        "attempting to send intial pageid to request worker"
    );

    pageid_send.send(first_id).await.unwrap();
    tracing::debug!("successfully send initial pageid");

    let mut jset = task::JoinSet::new();
    let mut task_map: HashMap<Id, String> = HashMap::new();

    let client: Arc<Client> = Arc::new(Client::builder().user_agent(USER_AGENT).build()?);
    tracing::trace!("successfully created request client");

    let batch_size: usize = 300;
    tracing::debug!(batch_size, "spawning writer task");
    let _writer_task = tokio::task::spawn(write_worker(pool, row_recv, batch_size));
    task_map.insert(_writer_task.id(), "WRITER_TASK".into());
    tracing::trace!(
        "successfully spawned writer task ID={} with batch size {}",
        _writer_task.id(),
        batch_size
    );

    for _i in 0..5 {
        let (recv, send) = (pageid_recv.clone(), id_bytes_send.clone());
        let client_ref: Arc<Client> = Arc::clone(&client);
        let _task = jset.spawn(request_worker(client_ref, recv, send));
        task_map.insert(_task.id(), format!("PROCESSING_TASK{}", _i));
        tracing::trace!(
            "successfully spawned request worker #{}, ID={}",
            _i,
            _task.id()
        );
    }
    drop(pageid_recv);
    drop(id_bytes_send);
    drop(client);
    tracing::trace!("extraneous request task reciever, sender, and client dropped");

    for _i in 0..5 {
        let (recv, send, send_write) =
            (id_bytes_recv.clone(), pageid_send.clone(), row_send.clone());
        let _task = jset.spawn_blocking(move || deserialize_worker(recv, send, send_write));
        task_map.insert(_task.id(), format!("REQUEST_TASK{}", _i));
        tracing::trace!(
            "successfully spawned response worker# {}, ID={}",
            _i,
            _task.id()
        );
    }
    drop(id_bytes_recv);
    drop(pageid_send);
    drop(row_send);
    tracing::trace!("extraneous reponse task reciever1, reciever2, and sender dropped");

    _writer_task.await??;

    let mut all_successful: bool = true;
    loop {
        match jset.join_next_with_id().await {
            Some(Ok((_id, result))) => match result {
                Ok(()) => {
                    tracing::info!(
                        "{} COMPLETED SUCCESSFULLY ",
                        task_map.get(&_id).unwrap_or(&"UNIDENTIFIED TASK".into())
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "{} FAILED WITH ERROR {}",
                        task_map.get(&_id).unwrap_or(&"UNIDENTIFIED TASK".into()),
                        e
                    );
                    all_successful = false;
                }
            },
            Some(Err(e)) => return Err(e)?,
            None => break,
        }
    }
    tracing::info!(
        SYNC_SUCCESS = all_successful,
        "FINISHED SYNCING DATABASE IN {}s",
        start_t.elapsed().as_secs_f32()
    );

    Ok(())
}

/// intial queries are built and send into the params channel
/// producer pulls a query from the channel
/// producer calls a request, and checks if the response container a continue value (signifying more results)
/// producer sends response bytes to consumer
/// consumer deserializes the response into a QueryResponse
/// consumer processes response and transforms into Row(s)
/// when a category is parsed, consumer passes the category's id back up to request worker
/// consumer sends Row(s) into row channel
/// writer recieves rows and prepares an insert statement
/// once batch size is reached (or on final flush) writer bulk writes to the db
async fn request_worker(
    client: Arc<Client>,
    recv: async_channel::Receiver<i32>,
    send: Sender<(i32, Vec<u8>)>,
) -> Result<()> {
    use tokio::time;
    loop {
        match time::timeout(time::Duration::from_secs(5), recv.recv()).await {
            Ok(Ok(id)) => {
                let mut params: PARAMS<Query> =
                    super::get::get_category_members_sync_params(id.clone())
                        .expect("get_category_members_sync_params failed");
                let mut more_results: bool = true;

                while more_results {
                    let resp: Response = client
                        .get(BASE)
                        .query(&params)
                        .send()
                        .await
                        .expect("request failed");

                    resp.error_for_status_ref()
                        .expect("response has a error status code");

                    let b: Vec<u8> = resp
                        .bytes()
                        .await
                        .expect("failed to convert response bytes into QueryResponse")
                        .to_vec();

                    match from_slice::<Continue>(&b).map(|c| c.into_tuple()) {
                        Ok(Some((ck, cv))) => {
                            params.update_continue(ck, cv);
                        }
                        _ => {
                            more_results = false;
                        }
                    }

                    send.try_send((id, b)).unwrap(); // send owned bytes into producer-consumer channel
                }
            }
            Err(_) | Ok(Err(_)) => break,
        }
    }
    drop(send);

    Ok(())
}

/// Breaks a single QueryResponse into ImageCategoryRows and SubCategoryRows and sends back any parsed category id's to the request worker
fn process_response(
    resp_bytes: Vec<u8>,
    resp_id: i32,
    send_id: async_channel::Sender<i32>,
    send_row: async_channel::Sender<Row>,
) -> Result<()> {
    use rayon::prelude::*;
    let resp: QueryResponse = from_slice(&resp_bytes[..])?;
    resp.results.into_par_iter().for_each(|qr| match qr.ns {
        NAMESPACE::CATEGORY => {
            send_id.send_blocking(qr.pageid.clone()).unwrap();
            send_row
                .send_blocking(Row::SubCategory(SubCategoryRow {
                    category_id: resp_id,
                    subcategory_id: qr.pageid,
                }))
                .expect("failed to send SubCategory row to writer");
            send_row
                .send_blocking(Row::Categories(qr.try_into().expect(
                    "failed to convert category result into CategoriesRow, missing category info",
                )))
                .expect("failed to send Categories row to writer");
        }
        NAMESPACE::FILE => {
            send_row
                .send_blocking(Row::ImageCategory(ImageCategoryRow {
                    image_id: qr.pageid.clone(),
                    category_id: resp_id,
                }))
                .expect("failed to send ImageCategoryRow to writer");
            send_row
                .send_blocking(Row::Images(qr.try_into().expect(
                    "failed to convert image result into ImagesRow, missing imageinfo",
                )))
                .expect("failed to send ImagesRow to writer");
        }
        _ => (),
    });

    Ok(())
}

/// this guy's doing a lot of work, here's an overview:
///
/// cm == CategoryMembers -- the worker processes API responses from requests that query
/// the categorymembers generator and transforms them to be written to the database.
///
/// The worker recieves the pageid of the category whose members were returned
/// in the API request; the worker also recieves the raw response bytes
///
/// The worker deserializes the response bytes into a QueryResponse object,
/// and iterates over each contained QueryResult
///
/// Each QueryResult is either transformed into a row of CATEGORIES TABLE and SUBCATEGORIES TABLE or
/// into a row of IMAGES TABLE and IMAGE_CATEGORY TABLE based on either being within the 'category' or 'file' namespace
///
/// If the QueryResult is a category, its pageid is passed back up to the request worker, who will eventually recieve
/// that pageid and query the categorymembers generator for that category
///
/// The rows are then sent into the writer channel, see the write_worker() function
fn deserialize_worker(
    recv: Receiver<(i32, Vec<u8>)>,
    send: async_channel::Sender<i32>,
    send_write: async_channel::Sender<Row>,
) -> Result<()> {
    let timeout = tokio::time::Duration::from_secs(10);
    loop {
        match recv.recv_timeout(timeout) {
            Ok((id, bytes)) => {
                let (send_id, send_row) = (send.clone(), send_write.clone());
                process_response(bytes, id, send_id, send_row).unwrap();
            }
            Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => panic!("response worker timed out"),
        }
    }
    drop(recv); // just because this branch needs pruned, doesn't mean there aren't more responses to process
    drop(send_write);
    drop(send);
    Ok(())
}
#[tracing::instrument]
async fn write_worker(
    pool: SqlitePool,
    recv: async_channel::Receiver<Row>,
    batch_size: usize,
) -> Result<()> {
    // ---------------------
    // BEGIN SETUP
    // counters keep track of transaction information for caching
    // tx begins intial transaction
    //
    let (mut subcat_rows, mut imgcat_rows, mut cat_rows, mut img_rows) = (
        Vec::<SubCategoryRow>::new(),
        Vec::<ImageCategoryRow>::new(),
        Vec::<CategoriesRow>::new(),
        Vec::<ImagesRow>::new(),
    );

    let mut img_q: QueryBuilder<Sqlite> = QueryBuilder::new(
        "INSERT OR IGNORE INTO  images(id, title, url_, size_, width, height, timestamp_) ",
    );
    let mut cat_q: QueryBuilder<Sqlite> =
        QueryBuilder::new("INSERT OR IGNORE INTO categories(id, title, subcats, files) ");
    let mut imgcat_q: QueryBuilder<Sqlite> =
        QueryBuilder::new("INSERT OR IGNORE INTO image_categories(image_id, category_id) ");
    let mut subcat_q: QueryBuilder<Sqlite> =
        QueryBuilder::new("INSERT OR IGNORE INTO subcategories(parent_id, child_id) ");

    let mut tx = pool.begin().await?;

    //
    // END SETUP
    // ---------------------
    // BEGIN ROW INSERTION LOOP
    // loop over the row reciever channel until dropped
    // pattern match the row and call the corresponding write helper
    // update the appropriate counter map entry with the return value of the write helper
    // commit current transaction when map['insert_count'] >= batch_size
    // and initialize new transaction
    //
    while let Ok(row) = recv.recv().await {
        match row {
            Row::Images(img) => {
                img_rows.push(img);
            }
            Row::Categories(cat) => {
                cat_rows.push(cat);
            }
            Row::ImageCategory(icat) => {
                imgcat_rows.push(icat);
            }
            Row::SubCategory(scat) => {
                subcat_rows.push(scat);
            }
        }
        if img_rows.len() >= batch_size {
            img_q.reset();
            let q = img_q
                .push_values(img_rows.into_iter(), |mut b, img| img.into_bind_values(b))
                .build();
            q.execute(&mut *tx).await?;
            img_rows = vec![];
        }
        if cat_rows.len() >= batch_size {
            cat_q.reset();
            let q = cat_q
                .push_values(cat_rows.into_iter(), |mut b, cat| cat.into_bind_values(b))
                .build();
            q.execute(&mut *tx).await?;
            cat_rows = vec![];
        }
        if imgcat_rows.len() >= batch_size {
            img_q.reset();
            let q = imgcat_q
                .push_values(imgcat_rows.into_iter(), |mut b, icat| {
                    icat.into_bind_values(b)
                })
                .build();
            q.execute(&mut *tx).await?;
            imgcat_rows = vec![];
        }
        if subcat_rows.len() >= batch_size {
            subcat_q.reset();
            let q = subcat_q
                .push_values(subcat_rows.into_iter(), |mut b, cat| {
                    cat.into_bind_values(b)
                })
                .build();
            q.execute(&mut *tx).await?;
            subcat_rows = vec![];
        }
    }
    //
    // END ROW INSERTION LOOP
    // ---------------------
    // BEGIN FINAL TRANSACTION
    // commit all rows remaining in the buffer
    // and update the transaction summary
    //
    if !img_rows.is_empty() {
        img_q.reset();
        let q = img_q
            .push_values(img_rows.into_iter(), |mut b, img| img.into_bind_values(b))
            .build();
        q.execute(&mut *tx).await?;
    }
    if !cat_rows.is_empty() {
        cat_q.reset();
        let q = cat_q
            .push_values(cat_rows.into_iter(), |mut b, cat| cat.into_bind_values(b))
            .build();
        q.execute(&mut *tx).await?;
    }
    if !imgcat_rows.is_empty() {
        img_q.reset();
        let q = imgcat_q
            .push_values(imgcat_rows.into_iter(), |mut b, cat| {
                cat.into_bind_values(b)
            })
            .build();
        q.execute(&mut *tx).await?;
    }

    if !subcat_rows.is_empty() {
        subcat_q.reset();
        let q = subcat_q
            .push_values(subcat_rows.into_iter(), |mut b, cat| {
                cat.into_bind_values(b)
            })
            .build();
        q.execute(&mut *tx).await?;
    }

    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bin_modules::get;
    use better_tracing::{Registry, fmt, prelude::*};
    use crossbeam_channel::{Receiver, Sender, unbounded};
    use std::fs;
    use std::{path::PathBuf, str::FromStr};
    use tokio::time;
    const ROOT_ID: i32 = 364;
    const ARMOR_CAT_ID: i32 = 33671;
    const ROOT_MEMBERS: [i32; 5] = [363, 369, 31716, 375, 510];

    fn remove_dev_db() {
        let path = PathBuf::from_str("data/dev.db").unwrap();
        if fs::exists(&path).unwrap() {
            let _ = fs::remove_file(path);
        }
    }

    #[sqlx::test]
    async fn test_migrations() {
        let mut conn = sqlx::SqlitePool::connect("data/dev.db")
            .await
            .expect("failed to create SqlitePool");
        run_migrations(&mut conn).await.expect("migrations failed");
    }

    #[tokio::test]
    async fn test_sync() {
        let stdout = fmt::layer().with_test_writer().pretty();
        let sub = Registry::default().with(stdout);
        tracing::subscriber::set_global_default(sub).unwrap();
        let handle = tokio::task::spawn(sync("data/dev.db", None));
        let r = handle.await.unwrap();
        dbg!(&r);

        assert!(r.is_ok())
    }

    #[tokio::test]
    async fn test_request_worker() {
        let now = tokio::time::Instant::now();
        let (send, recv) = async_channel::unbounded::<i32>();
        let (send2, recv2) = unbounded::<(i32, Vec<u8>)>();
        let client: Arc<Client> =
            Arc::new(Client::builder().user_agent(USER_AGENT).build().unwrap());
        let h = tokio::spawn(request_worker(client, recv, send2));

        for id in ROOT_MEMBERS {
            send.send(id).await.unwrap();
        }

        drop(send);
        h.await.unwrap().unwrap();

        while let Ok((id, _bytes)) = recv2.recv() {
            dbg!(format!("fetched page #{}", id));
        }

        let el = now.elapsed();
        dbg!(format!(
            "request worker took {} seconds to run",
            el.as_secs_f64()
        ));
    }

    #[tokio::test]
    async fn test_empty_request_worker() {
        let (send, recv) = async_channel::unbounded::<i32>();
        let (send2, recv2) = unbounded::<(i32, Vec<u8>)>();
        let client: Arc<Client> =
            Arc::new(Client::builder().user_agent(USER_AGENT).build().unwrap());
        let h = tokio::spawn(request_worker(client, recv, send2));

        drop(send);

        h.await.unwrap().unwrap();

        while let Ok((id, _bytes)) = recv2.recv() {
            dbg!(id);
        }
    }

    #[tokio::test]
    async fn test_high_traffic_req_worker() {
        let (send, recv) = async_channel::unbounded::<i32>();
        let (send2, recv2) = unbounded::<(i32, Vec<u8>)>();
        for id in 0..500 {
            send.send(id).await.unwrap();
        }

        let mut jset: tokio::task::JoinSet<Result<()>> = tokio::task::JoinSet::new();
        let client: Arc<Client> =
            Arc::new(Client::builder().user_agent(USER_AGENT).build().unwrap());

        for _ in 0..5 {
            let (recv_copy, send_copy) = (recv.clone(), send2.clone());
            jset.spawn(request_worker(Arc::clone(&client), recv_copy, send_copy));
        }
        drop(client);

        drop(send);
        drop(recv);
        drop(send2);

        let r = jset.join_all().await;

        while let Ok((id, _bytes)) = recv2.recv() {
            if id % 100 == 0 {
                dbg!(id);
            }
        }

        assert!(r.iter().all(|r| r.is_ok()))
    }

    #[tokio::test]
    async fn test_process_and_write() {
        let (send_, recv_) = unbounded::<(i32, Vec<u8>)>();
        let (send_id, recv_id) = async_channel::unbounded::<i32>();
        let (send_row, recv_row) = async_channel::unbounded::<Row>();

        let root_params: PARAMS<Query> = get::get_category_members_sync_params(ROOT_ID).unwrap();
        let root_bytes: Vec<u8> = Client::new()
            .get(BASE)
            .query(&root_params)
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap()
            .to_vec();

        let opts = SqliteConnectOptions::new()
            .foreign_keys(false)
            .create_if_missing(true)
            .filename("data/dev.db");
        let pool = SqlitePoolOptions::new().connect_lazy_with(opts);

        let handle = task::spawn(write_worker(pool, recv_row, 300));

        let handle2 =
            tokio::task::spawn_blocking(move || deserialize_worker(recv_, send_id, send_row));

        send_.send((ROOT_ID, root_bytes)).unwrap();

        drop(send_);

        handle2.await.unwrap().unwrap();
        dbg!("response handle joined");
        handle.await.unwrap().unwrap();
        dbg!("writer handle joined");
        while let Ok(id) = recv_id.recv().await {
            dbg!(format!("recieved pageid: {}", id));
        }
    }

    #[tokio::test]
    async fn test_nested_request_worker() {
        let (send, recv) = async_channel::unbounded::<i32>();
        let (send2, recv2) = unbounded::<(i32, Vec<u8>)>();
        let (send_row, recv_row) = async_channel::unbounded::<Row>();
        send.send(ARMOR_CAT_ID).await.unwrap();
        let mut jset: tokio::task::JoinSet<Result<()>> = tokio::task::JoinSet::new();
        // let mut resp_jset: tokio::task::JoinSet<Result<()>> = tokio::task::JoinSet::new();

        let client: Arc<Client> =
            Arc::new(Client::builder().user_agent(USER_AGENT).build().unwrap());

        for _ in 0..2 {
            let (recv_copy, send_copy) = (recv.clone(), send2.clone());
            jset.spawn(request_worker(Arc::clone(&client), recv_copy, send_copy));
        }
        drop(client);

        for _ in 0..2 {
            let (recv_copy, send_copy, send_row_copy) =
                (recv2.clone(), send.clone(), send_row.clone());
            jset.spawn_blocking(move || deserialize_worker(recv_copy, send_copy, send_row_copy));
        }

        drop(send);
        drop(send2);
        drop(send_row);

        let r = jset.join_all().await;
        // let resp_r = resp_jset.join_all().await;

        let timeout = time::Duration::from_secs(10);

        loop {
            match tokio::time::timeout(timeout, recv_row.recv()).await {
                Ok(Ok(r)) => {
                    dbg!(r);
                }
                Ok(Err(_)) => break,
                Err(_) => panic!("timeout panic"),
            }
        }

        assert!(r.iter().all(|r| r.is_ok()));
    }
}
