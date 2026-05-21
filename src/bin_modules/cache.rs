use serde::{Deserialize, Serialize};
use std::time;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Cache {
    pub last_sync_at: Option<time::SystemTime>,
    pub last_sync_rows_written: Option<u64>,
}
