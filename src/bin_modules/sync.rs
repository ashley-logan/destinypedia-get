use super::database::rows::{
    CategoriesRow, ImageCategoryRow, ImagesRow, Row, SubCategoryRow, into_categories_row,
    into_images_row,
};
use super::database::write::{dispatch_row_writer, create_tables};
use crossbeam_channel::{Receiver, Sender, bounded};
use destinypedia::NAMESPACE;
use destinypedia::request::{PARAMS, Query};
use super::Result;
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
use std::thread::JoinHandle;
use tokio::task;

const USER_AGENT: &'static str = "DESTINY_FETCHER";
const BASE: &'static str = "https://www.destinypedia.com/api.php";
const CATEGORY_IMAGES_ID: u32 = 364;



pub async fn sync(mut data_dir: path::PathBuf) -> Result<()> {
    let start_t = tokio::time::Instant::now();

    data_dir.push(path::Path::new("destiny-fetch.db"));
    let conn = Connection::open(&data_dir)?;

    create_tables(data_dir)?;


    let (pageid_send, pageid_recv) = bounded::<u32>(1000);
    let (id_bytes_send, id_bytes_recv) = bounded::<(u32, Vec<u8>)>(1000);
    let (row_send, row_recv) = bounded::<Row>(1000);
    let jhandle: JoinHandle<Result<HashMap<String, usize>>> = std::thread::spawn(move || dispatch_row_writer(&conn, row_recv, 350).map_err(|e| e.into()));

    let mut jset: task::JoinSet<Result<()>> = task::JoinSet::new();

    (0..5).for_each(|_| {
        let (recv, send, send_write) =
            (id_bytes_recv.clone(), pageid_send.clone(), row_send.clone());
        jset.spawn(cm_response_worker(recv, send, send_write));
    });

    pageid_send.send(CATEGORY_IMAGES_ID).unwrap();

    (0..5).for_each(|_| {
        let (recv, send) = (pageid_recv.clone(), id_bytes_send.clone());
        jset.spawn(cm_request_worker(recv, send));
    });

    let results: Result<Vec<()>> = jset.join_all().await.into_iter().collect();
    let write_result: std::result::Result<Result<HashMap<String, usize>>, Box<dyn std::any::Any + Send>> = jhandle.join();

    let insertion_summary = match (results, write_result) {
        (Ok(_), Ok(Ok(map))) => map,
        (Ok(_), Ok(Err(e))) => Err(e)?,
        (Ok(_), Err(_)) => Err(super::DestinyFetchError::DatabaseErr)?,
        (Err(e), _) => Err(e)? 
    };

    dbg!(format!("sync finished executing in {} seconds", start_t.elapsed().as_secs_f32()));

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
pub async fn cm_request_worker(
    recv: Receiver<u32>,
    send: Sender<(u32, Vec<u8>)>,
) -> Result<()> {
    let client: Client = Client::builder().user_agent(USER_AGENT).build()?;

    while let Ok(id) = recv.recv() {
        let mut params: PARAMS<Query> = super::get::get_category_members_sync_params(id.clone())?;
        let mut more_results: bool = true;

        while more_results {
            let resp: Response = client.get(BASE).query(&params).send().await?;

            resp.error_for_status_ref()?;

            let b: Vec<u8> = resp.bytes().await?.to_vec();

            match from_slice::<Continue>(&b).map(|c| c.into_tuple()) {
                Ok(Some((ck, cv))) => {
                    params.update_continue(ck, cv);
                }
                _ => {
                    more_results = false;
                }
            }

            send.send((id, b)).unwrap(); // send owned bytes into producer-consumer channel
        }
    }
    drop(send);

    Ok(())
}


/// Breaks a single QueryResponse into ImageCategoryRows and SubCategoryRows and sends back any parsed category id's to the request worker
fn process_response(resp_bytes: Vec<u8>, resp_id: u32, has_categories: Arc<AtomicBool>, send_id: Sender<u32>, send_row: Sender<Row>) -> Result<()> {
    use rayon::prelude::*;
    let resp: QueryResponse = from_slice(&resp_bytes[..])?;
    resp.results.into_par_iter().for_each(|qr| match qr.ns {
        NAMESPACE::CATEGORY => {
            has_categories.store(true, Ordering::Relaxed);
            send_id.send(qr.pageid.clone()).unwrap();
            send_row
                .send(Row::SubCategory(SubCategoryRow {
                    id: resp_id,
                    subcategory_id: qr.pageid,
                }))
                .expect("failed to send SubCategory row to writer");
            send_row
                .send(Row::Categories(into_categories_row(qr).expect(
                    "failed to convert category result into CategoriesRow",
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
                .send(Row::Images(
                    into_images_row(qr)
                        .expect("failed to convert image result into ImagesRow"),
                ))
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
pub(crate) async fn cm_response_worker(
    recv: Receiver<(u32, Vec<u8>)>,
    send: Sender<u32>,
    send_write: Sender<Row>,
) -> Result<()> {
    let child_categories: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let mut handle: task::JoinSet<Result<()>> = tokio::task::JoinSet::new();
    while let Ok((id, bytes)) = recv.recv() {
        child_categories.store(false, Ordering::Relaxed);
        let (send_id, send_row) = (send.clone(), send_write.clone());
        let has_categories = child_categories.clone();
        handle.spawn_blocking(move || process_response(bytes, id, has_categories, send_id, send_row));
        if child_categories.load(Ordering::Relaxed) == false {
            drop(recv);
            break;
        }
    }

    drop(send_write);
    drop(send);
    let r: Result<Vec<()>> = handle.join_all().await.into_iter().collect();
    r.map(|_| ())
}

#[cfg(test)]
mod tests {
    use std::{path::{Path, PathBuf}, str::FromStr};
    use crate::bin_modules::get;
    use destinypedia::request::*;
    use std::{fs, os};
    use super::*;
    use crossbeam_channel::{Receiver, Sender, unbounded};
    const ROOT_ID: u32 = 364;
    const ROOT_SUBCATS: u16 = 5;
    const ROOT_MEMBERS: [u32; 5] = [363, 369, 31716, 375, 510];

    fn remove_dev_db() {
        let path = PathBuf::from_str("data/dev.db").unwrap();
        if fs::exists(&path).unwrap() {
            let _ = fs::remove_file(path);
        }
    }

    #[tokio::test]
    async fn test_request_worker() {
        let now = tokio::time::Instant::now();
        let (send, recv) = unbounded::<u32>();
        let (send2, recv2) = unbounded::<(u32, Vec<u8>)>();
        let h = tokio::spawn(cm_request_worker(recv, send2));

        for id in ROOT_MEMBERS {
            send.send(id).unwrap();
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
        let (send, recv) = unbounded::<u32>();
        let (send2, recv2) = unbounded::<(u32, Vec<u8>)>();
        let h = tokio::spawn(cm_request_worker(recv, send2));

        drop(send);

        h.await.unwrap().unwrap();

        while let Ok((id, _bytes)) = recv2.recv() {
            dbg!(format!("fetched page #{}", id));
        }
    }

    #[tokio::test]
    async fn test_high_traffic_req_worker() {
        let (send, recv) = unbounded::<u32>();
        let (send2, recv2) = unbounded::<(u32, Vec<u8>)>();
        (0..500_u32).for_each(|id| {
            send.send(id).unwrap();
        });
        let mut jset: tokio::task::JoinSet<Result<()>> = tokio::task::JoinSet::new();

        for _ in 0..10 {
            let (recv_copy, send_copy) = (recv.clone(), send2.clone());
            jset.spawn(cm_request_worker(recv_copy, send_copy));
        }

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
        let (send_id, recv_id) = unbounded::<u32>();
        let (send_row, recv_row) = unbounded::<Row>();

        let root_params: PARAMS<Query> = get::get_category_members_sync_params(ROOT_ID).unwrap();
        let root_bytes: Vec<u8> = Client::new().get(BASE).query(&root_params).send().await.unwrap().bytes().await.unwrap().to_vec();
        
        create_tables(PathBuf::from_str("data/dev.db").unwrap()).unwrap();
        let conn = Connection::open("data/dev.db").unwrap();

        let handle = std::thread::spawn(move || dispatch_row_writer(&conn, recv_row, 100));

        let async_handle = tokio::task::spawn(cm_response_worker(recv_, send_id, send_row));

        send_.send((ROOT_ID, root_bytes)).unwrap();

        

        drop(send_);

        
        async_handle.await.unwrap().unwrap();
        dbg!("async handle joined");
        handle.join().unwrap().unwrap();
        dbg!("thread handle joined");
        while let Ok(id) = recv_id.recv() {
            dbg!(format!("recieved pageid: {}", id));
        }
    }
}
