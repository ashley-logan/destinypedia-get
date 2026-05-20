use super::Result;
use super::database::rows::{
    CategoriesRow, ImageCategoryRow, ImagesRow, Row, SubCategoryRow, into_categories_row,
    into_images_row,
};
use super::database::write::{create_tables, dispatch_row_writer};
use async_channel;
use crossbeam_channel::{Receiver, RecvTimeoutError, SendError, Sender, bounded, unbounded};
use destinypedia::NAMESPACE;
use destinypedia::request::{PARAMS, Query};
use destinypedia::response::{Continue, QueryResponse};
use dirs;
use reqwest::{Client, Response};
use rusqlite::Connection;
use serde_json::from_slice;
use std::collections::HashMap;
use std::path;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::task;

const USER_AGENT: &'static str = "DESTINY_FETCH";
const BASE: &'static str = "https://www.destinypedia.com/api.php";
const CATEGORY_IMAGES_ID: u32 = 364;

pub async fn sync(
    database: path::PathBuf,
    starting_pageid: Option<u32>,
) -> Result<HashMap<String, usize>> {
    let start_t = tokio::time::Instant::now();

    create_tables(&database)?;

    let (pageid_send, pageid_recv) = async_channel::bounded::<u32>(500);
    let (id_bytes_send, id_bytes_recv) = unbounded::<(u32, Vec<u8>)>();
    let (row_send, row_recv) = bounded::<Row>(500);

    let conn = Connection::open(&database)?;

    pageid_send
        .send(starting_pageid.unwrap_or(CATEGORY_IMAGES_ID))
        .await
        .unwrap();

    let mut jset = task::JoinSet::new();

    let writer_handle =
        tokio::task::spawn_blocking(move || dispatch_row_writer(conn, row_recv, 300).unwrap());

    let client: Arc<Client> = Arc::new(Client::builder().user_agent(USER_AGENT).build()?);

    for _ in 0..4 {
        let (recv, send) = (pageid_recv.clone(), id_bytes_send.clone());
        let client_ref: Arc<Client> = Arc::clone(&client);
        jset.spawn(cm_request_worker(client_ref, recv, send));
    }
    jset.spawn(cm_request_worker(client, pageid_recv, id_bytes_send));

    for _ in 0..4 {
        let (recv, send, send_write) =
            (id_bytes_recv.clone(), pageid_send.clone(), row_send.clone());
        jset.spawn_blocking(move || cm_response_worker(recv, send, send_write));
    }
    jset.spawn_blocking(move || cm_response_worker(id_bytes_recv, pageid_send, row_send));

    let _results = jset.join_all().await;
    let write_summary: HashMap<String, usize> = writer_handle.await.unwrap();

    dbg!(format!(
        "sync finished executing in {} seconds",
        start_t.elapsed().as_secs_f32()
    ));

    Ok(write_summary)
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
pub async fn cm_request_worker(
    client: Arc<Client>,
    recv: async_channel::Receiver<u32>,
    send: Sender<(u32, Vec<u8>)>,
) -> Result<()> {
    use tokio::time;
    loop {
        match time::timeout(time::Duration::from_secs(10), recv.recv()).await {
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
    resp_id: u32,
    send_id: async_channel::Sender<u32>,
    send_row: Sender<Row>,
) -> Result<()> {
    use rayon::prelude::*;
    let resp: QueryResponse = from_slice(&resp_bytes[..])?;
    resp.results.into_par_iter().for_each(|qr| match qr.ns {
        NAMESPACE::CATEGORY => {
            send_id.send_blocking(qr.pageid.clone()).unwrap();
            send_row
                .send(Row::SubCategory(SubCategoryRow {
                    id: resp_id,
                    subcategory_id: qr.pageid,
                }))
                .expect("failed to send SubCategory row to writer");
            send_row
                .send(Row::Categories(into_categories_row(qr).expect(
                    "failed to convert category result into CategoriesRow, missing category info",
                )))
                .expect("failed to send Categories row to writer");
        }
        NAMESPACE::FILE => {
            send_row
                .send(Row::ImageCategory(ImageCategoryRow {
                    image_id: qr.pageid.clone(),
                    category_id: resp_id,
                }))
                .expect("failed to send ImageCategoryRow to writer");
            send_row
                .send(Row::Images(into_images_row(qr).expect(
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
/// The rows are then sent into the writer channel, see the dispatch_row_writer() function
pub(crate) fn cm_response_worker(
    recv: Receiver<(u32, Vec<u8>)>,
    send: async_channel::Sender<u32>,
    send_write: Sender<Row>,
) -> Result<()> {
    let timeout = tokio::time::Duration::from_secs(30);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bin_modules::get;
    use crossbeam_channel::{Receiver, Sender, unbounded};
    use std::fs;
    use std::{path::PathBuf, str::FromStr};
    use tokio::time;
    const ROOT_ID: u32 = 364;
    const ARMOR_CAT_ID: u32 = 33671;
    const ROOT_MEMBERS: [u32; 5] = [363, 369, 31716, 375, 510];

    fn remove_dev_db() {
        let path = PathBuf::from_str("data/dev.db").unwrap();
        if fs::exists(&path).unwrap() {
            let _ = fs::remove_file(path);
        }
    }

    #[tokio::test]
    async fn test_sync() {
        remove_dev_db();

        let db = PathBuf::from_str("data/dev.db").unwrap();

        let map = sync(db, None).await.unwrap();

        dbg!(map);
    }

    #[tokio::test]
    async fn test_request_worker() {
        let now = tokio::time::Instant::now();
        let (send, recv) = async_channel::unbounded::<u32>();
        let (send2, recv2) = unbounded::<(u32, Vec<u8>)>();
        let client: Arc<Client> =
            Arc::new(Client::builder().user_agent(USER_AGENT).build().unwrap());
        let h = tokio::spawn(cm_request_worker(client, recv, send2));

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
        let (send, recv) = async_channel::unbounded::<u32>();
        let (send2, recv2) = unbounded::<(u32, Vec<u8>)>();
        let client: Arc<Client> =
            Arc::new(Client::builder().user_agent(USER_AGENT).build().unwrap());
        let h = tokio::spawn(cm_request_worker(client, recv, send2));

        drop(send);

        h.await.unwrap().unwrap();

        while let Ok((id, _bytes)) = recv2.recv() {
            dbg!(format!("fetched page #{}", id));
        }
    }

    #[tokio::test]
    async fn test_high_traffic_req_worker() {
        let (send, recv) = async_channel::unbounded::<u32>();
        let (send2, recv2) = unbounded::<(u32, Vec<u8>)>();
        for id in 0..500 {
            send.send(id).await.unwrap();
        }

        let mut jset: tokio::task::JoinSet<Result<()>> = tokio::task::JoinSet::new();
        let client: Arc<Client> =
            Arc::new(Client::builder().user_agent(USER_AGENT).build().unwrap());

        for _ in 0..10 {
            let (recv_copy, send_copy) = (recv.clone(), send2.clone());
            jset.spawn(cm_request_worker(Arc::clone(&client), recv_copy, send_copy));
        }
        drop(client);

        drop(send);
        drop(recv);
        drop(send2);

        let r = jset.join_all().await;

        while let Ok((id, _bytes)) = recv2.recv() {
            if id % 100 == 0 {
                dbg!(format!("fetched page {}", id));
            }
        }

        assert!(r.iter().all(|r| r.is_ok()))
    }

    #[tokio::test]
    async fn test_process_and_write() {
        remove_dev_db();
        let (send_, recv_) = unbounded::<(u32, Vec<u8>)>();
        let (send_id, recv_id) = async_channel::unbounded::<u32>();
        let (send_row, recv_row) = unbounded::<Row>();

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

        create_tables(PathBuf::from_str("data/dev.db").unwrap()).unwrap();
        let conn = Connection::open("data/dev.db").unwrap();

        let handle = std::thread::spawn(move || dispatch_row_writer(conn, recv_row, 100));

        let handle2 =
            tokio::task::spawn_blocking(move || cm_response_worker(recv_, send_id, send_row));

        send_.send((ROOT_ID, root_bytes)).unwrap();

        drop(send_);

        handle2.await.unwrap().unwrap();
        dbg!("async handle joined");
        handle.join().unwrap().unwrap();
        dbg!("thread handle joined");
        while let Ok(id) = recv_id.recv().await {
            dbg!(format!("recieved pageid: {}", id));
        }
    }

    #[tokio::test]
    async fn test_nested_request_worker() {
        let (send, recv) = async_channel::unbounded::<u32>();
        let (send2, recv2) = unbounded::<(u32, Vec<u8>)>();
        let (send_row, recv_row) = unbounded::<Row>();
        send.send(ARMOR_CAT_ID).await.unwrap();
        let mut jset: tokio::task::JoinSet<Result<()>> = tokio::task::JoinSet::new();
        // let mut resp_jset: tokio::task::JoinSet<Result<()>> = tokio::task::JoinSet::new();

        let client: Arc<Client> =
            Arc::new(Client::builder().user_agent(USER_AGENT).build().unwrap());

        for _ in 0..2 {
            let (recv_copy, send_copy) = (recv.clone(), send2.clone());
            jset.spawn(cm_request_worker(Arc::clone(&client), recv_copy, send_copy));
        }
        drop(client);

        for _ in 0..2 {
            let (recv_copy, send_copy, send_row_copy) =
                (recv2.clone(), send.clone(), send_row.clone());
            jset.spawn_blocking(move || cm_response_worker(recv_copy, send_copy, send_row_copy));
        }

        drop(send);
        drop(send2);
        drop(send_row);

        let r = jset.join_all().await;
        // let resp_r = resp_jset.join_all().await;

        let timeout = time::Duration::from_secs(10);

        while let Ok(row) = recv_row.recv_timeout(timeout) {
            dbg!(row);
        }

        assert!(r.iter().all(|r| r.is_ok()));
        // assert!((resp_r.iter().all(|r| r.is_ok())));
    }
}
