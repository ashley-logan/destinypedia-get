use crate::database::{
    CategoriesRow, ImageCategoryRow, ImagesRow, Row, SubCategoryRow, into_categories_row,
    into_images_row,
};
use crate::{QueryResponse, deserialize::QueryResult, models::NAMESPACE};
use crossbeam_channel::{Receiver, Sender};

use serde_json::from_slice;

pub fn cm_response_worker(
    recv: Receiver<(u32, Vec<u8>)>,
    send: Sender<u32>,
    send_write: Sender<Row>,
) {
    use rayon::prelude::*;
    while let Ok((id, bytes)) = recv.recv() {
        let resp: QueryResponse = from_slice(&bytes[..]).unwrap();
        resp.results.into_par_iter().for_each(|qr| match qr.ns {
            NAMESPACE::CATEGORY => {
                send.send(qr.pageid.clone()).unwrap();
                send_write
                    .send(Row::SubCategory(SubCategoryRow {
                        id,
                        subcategory_id: qr.pageid,
                    }))
                    .expect("failed to send SubCategory row to writer");
                send_write
                    .send(Row::Categories(into_categories_row(qr).expect(
                        "failed to convert category result into CategoriesRow",
                    )))
                    .expect("failed to send Categories row to writer");
            }
            NAMESPACE::FILE => {
                send_write
                    .send(Row::ImageCategory(ImageCategoryRow {
                        image_id: qr.pageid.clone(),
                        category_id: id,
                    }))
                    .expect("failed to send ImageCategoryRow to writer");
                send_write
                    .send(Row::Images(
                        into_images_row(qr).expect("failed to convert image result into ImagesRow"),
                    ))
                    .expect("failed to send ImagesRow to writer");
            }
            _ => panic!("result sent to cm_response_worker was neither a file nor a category"),
        });
    }
}
