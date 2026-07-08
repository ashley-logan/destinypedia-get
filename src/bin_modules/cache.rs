use super::database::rows::ImagesRow;
use super::{DestinyFetchError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, os};

pub const CACHE_FILE: &str = "destiny_fetch_cache.json";

#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct Cache {
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_sync_rows_written: Option<u64>,
    #[serde(flatten)]
    pub cached_searches: HashMap<String, Vec<ImagesRow>>,
}

impl Cache {
    fn default_path() -> Result<PathBuf> {
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
    pub fn open_or_create(path: Option<PathBuf>) -> Result<Cache> {
        let path = path.unwrap_or(Self::default_path()?);
        match fs::read(path) {
            Ok(v) => serde_json::from_slice(&v[..]).map_err(|_| DestinyFetchError::CachePathErr),
            Err(_) => Ok(Cache::new()),
        }
    }
    pub fn open(path: Option<PathBuf>) -> Result<Cache> {
        let path = path.unwrap_or(Self::default_path()?);
        let v: Vec<u8> = fs::read(path)?;
        serde_json::from_slice(&v[..]).map_err(|_| DestinyFetchError::CachePathErr)
    }
    pub fn len(&self) -> usize {
        self.cached_searches.len()
    }
    pub fn write_images(&mut self, images: Vec<ImagesRow>, path: Option<PathBuf>) -> Result<()> {
        self.cached_searches.insert(self.new_save_name(), images);
        let path = path.unwrap_or(Self::default_path()?);
        serde_json::to_writer_pretty(fs::File::open(path)?, self)?;
        Ok(())
    }
    pub fn write_images_with_name(
        &mut self,
        name: String,
        images: Vec<ImagesRow>,
        path: Option<PathBuf>,
    ) -> Result<()> {
        self.cached_searches.insert(name, images);
        let path = path.unwrap_or(Self::default_path()?);
        serde_json::to_writer_pretty(fs::File::open(path)?, self)?;
        Ok(())
    }
    pub fn read_images(&self, name: impl AsRef<str>) -> Option<&Vec<ImagesRow>> {
        self.cached_searches.get(name.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, NaiveDateTime};

    fn remove_cache(path: Option<PathBuf>) {
        let path = path.unwrap_or(Cache::default_path().expect("unable to get default cache path"));
        if fs::exists(&path).expect("unable to check filesystem") {
            fs::remove_file(path).expect("unable to remove cache")
        }
    }

    #[test]
    fn test_open_new_cache() {
        remove_cache(None);
        let exp = Cache::new();
        let test = Cache::open_or_create(None).expect("failed to open Cache");
        assert_eq!(exp, test)
    }

    #[test]
    fn test_get_path() {
        let path = Cache::default_path().expect("unable to get cache path");
        dbg!(path);
    }

    #[test]
    fn test_store_images() {
        remove_cache(None);
        let mut cache = Cache::new();
        let now: DateTime<Utc> = Local::now().to_utc();
        cache.last_sync_at = Some(now);
        cache.last_sync_rows_written = Some(10);
        cache.cached_searches = HashMap::new();
        let ser = serde_json::to_string_pretty(&cache).expect("unable to serialize cache");
        dbg!(ser);
    }
    #[test]
    fn test_store_images_null() {
        remove_cache(None);
        let mut cache = Cache::new();
        let ser = serde_json::to_string_pretty(&cache).expect("unable to serialize cache");
        dbg!(ser);
    }
}
