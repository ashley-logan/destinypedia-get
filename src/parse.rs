use crate::QueryResponse;
use crossbeam_channel::Receiver;
use rayon::iter::{self, IntoParallelRefIterator, ParallelIterator};
use serde_json::from_slice;

// pub fn process_response(rc: Receiver<Vec<u8>>) {
//     while let Ok(bytes) = rc.recv() {
//         match from_slice::<QueryResponse>(&bytes[..]) {
//             Ok(resp) => {
//                 resp.results
//                 .par_iter()
//                 .map(|&result| );
//             }
//             Err(_) => todo!(),
//         }
//     }
// }
