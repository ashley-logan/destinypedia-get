use crate::{
    PARAMS, QueryResponse, SubCategoryRow,
    serialize::{GcmIdentifier, Generator, Limit, Query},
};
use crossbeam_channel::{Receiver, Sender, bounded};
use rayon::iter::{self, IntoParallelRefIterator, ParallelIterator};
use serde_json::from_slice;

pub fn dispatch_response(recv: Receiver<Vec<u8>>) {
    while let Ok(bytes) = recv.recv() {
        if let Ok(resp) = from_slice::<QueryResponse>(&bytes[..]) {
            todo!("process function for each type")
        }
    }
}

/// SUBCATEGORY TABLE
/// id: category's page id
/// subcat_id: subcategory's page id
pub(crate) fn process_category_members(id: u32, resp: &QueryResponse, send: Sender<PARAMS<Query>>) {
    if let Some(ids) = &resp.pageids {
        let rows: Vec<SubCategoryRow> = ids
            .iter()
            .map(|subid| SubCategoryRow::from((id, *subid)))
            .collect();

        // todo!("pass rows to writer");

        for &id in ids {
            let params: PARAMS<Query> = PARAMS::build()
                .with_generator(Generator::categorymembers_with(
                    GcmIdentifier::GcmPageid(id),
                    None,
                    Some(Limit::Max),
                ))
                .with_indexpageids(true)
                .build()
                .unwrap();

            send.send(params).unwrap();
        }
    }
}
