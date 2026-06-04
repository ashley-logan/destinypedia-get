use super::database::rows::ImagesRow;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const CACHE_FILE: &str = "destiny_fetch_cache.json";

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Cache {
    pub last_sync_at: Option<NaiveDateTime>,
    pub last_sync_rows_written: Option<u64>,
    #[serde(flatten)]
    pub cached_searches: HashMap<String, Vec<ImagesRow>>,
}
