use super::database::rows::ImagesRow;
use super::{DestinyFetchError, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, os};

pub const CACHE_FILE: &str = "destiny_fetch_cache.json";

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Cache {
    pub last_sync_at: Option<NaiveDateTime>,
    pub last_sync_rows_written: Option<u64>,
    #[serde(flatten)]
    pub cached_searches: HashMap<String, Vec<ImagesRow>>,
}

impl Cache {
    fn path() -> Result<PathBuf> {
        let mut path = dirs::cache_dir().ok_or(DestinyFetchError::CachePathErr)?;
        path.push(CACHE_FILE);
        Ok(path)
    }
    pub fn new_save_name(&self) -> String {
        format!("stored_query_{}", self.len() + 1)
    }
    pub fn new() -> Self {
        Cache {
            last_sync_at: None,
            last_sync_rows_written: None,
            cached_searches: HashMap::new(),
        }
    }
    pub fn open() -> Result<Cache> {
        match fs::read(Self::path()?) {
            Ok(v) => serde_json::from_slice(&v[..]).map_err(|_| DestinyFetchError::CachePathErr),
            Err(_) => Ok(Cache::new()),
        }
    }
    pub fn len(&self) -> usize {
        self.cached_searches.len()
    }
    pub fn store_images(&mut self, save_name: String, images: Vec<ImagesRow>) -> Result<()> {
        self.cached_searches.insert(save_name, images);
        serde_json::to_writer_pretty(fs::File::open(Self::path()?)?, self)?;
        Ok(())
    }
}
