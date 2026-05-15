use crate::database::{
    CategoriesRow, ImageCategoryRow, ImagesRow, Row, SubCategoryRow, dispatch_row_writer,
    into_categories_row, into_images_row,
};
use crate::{Continue, PARAMS, Query, QueryResponse, Result, get, models::NAMESPACE};
use crossbeam_channel::{Receiver, Sender, bounded};
use dirs;
use reqwest::{Client, Response};
use rusqlite::Connection;
use serde_json::from_slice;
use std::path;

const USER_AGENT: &'static str = "DESTINY_FETCHER";
const BASE: &'static str = "https://www.destinypedia.com/api.php";
const CATEGORY_IMAGES_ID: u32 = 364;

pub async fn sync(mut data_dir: path::PathBuf) -> Result<()> {
    data_dir.push(path::Path::new("destiny-fetch.db"));
    let conn = Connection::open(data_dir)?;

    let (pageid_send, pageid_recv) = bounded::<u32>(1000);
    let (id_bytes_send, id_bytes_recv) = bounded::<(u32, Vec<u8>)>(1000);
    let (row_send, row_recv) = bounded::<Row>(1000);

    std::thread::scope(|s| {
        s.spawn(move || {
            dispatch_row_writer(&conn, row_recv, 350_usize);
        });
    });

    (0..5).for_each(|_| {
        let (recv, send, send_write) =
            (id_bytes_recv.clone(), pageid_send.clone(), row_send.clone());
        tokio::spawn(async move { cm_response_worker(recv, send, send_write).await });
    });

    pageid_send.send(CATEGORY_IMAGES_ID).unwrap();

    (0..5).for_each(|_| {
        let (recv, send) = (pageid_recv.clone(), id_bytes_send.clone());
        tokio::spawn(async move {
            cm_request_worker(recv, send).await;
        });
    });

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
pub async fn cm_request_worker(recv: Receiver<u32>, send: Sender<(u32, Vec<u8>)>) -> Result<()> {
    let client: Client = Client::builder().user_agent(USER_AGENT).build()?;

    for id in recv.recv().into_iter() {
        let mut params: PARAMS<Query> = get::get_category_members_sync_params(id.clone())?;
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
) {
    use rayon::prelude::*;
    while let Ok((id, bytes)) = recv.recv() {
        let send_id: Sender<u32> = send.clone();
        let send_row: Sender<Row> = send_write.clone();
        tokio::task::spawn_blocking(move || {
            let resp: QueryResponse = from_slice(&bytes[..]).unwrap();

            resp.results.into_par_iter().for_each(|qr| match qr.ns {
                NAMESPACE::CATEGORY => {
                    send_id.send(qr.pageid.clone()).unwrap();
                    send_row
                        .send(Row::SubCategory(SubCategoryRow {
                            id,
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
                            category_id: id,
                        }))
                        .expect("failed to send ImageCategoryRow to writer");
                    send_row
                        .send(Row::Images(
                            into_images_row(qr)
                                .expect("failed to convert image result into ImagesRow"),
                        ))
                        .expect("failed to send ImagesRow to writer");
                }
                _ => panic!("result sent to cm_response_worker was neither a file nor a category"),
            });
        })
        .await
        .unwrap();
    }
}
