use super::database::rows::ImagesRow;
use super::{DestinyFetchError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, os};

pub const CACHE_FILE: &str = "destiny_fetch_cache.json";

#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct Cachedata {
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_sync_rows_written: Option<u64>,
    #[serde(flatten)]
    pub cached_searches: HashMap<String, Vec<ImagesRow>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cache {
    pub data: Cachedata,
    pub path: PathBuf,
}

impl Cache {
    fn default_path() -> Result<PathBuf> {
        let mut path = dirs::cache_dir().ok_or(DestinyFetchError::CachePathErr)?;
        path.push(CACHE_FILE);
        Ok(path)
    }
    pub fn new() -> Result<Self> {
        let path = Self::default_path()?;
        Ok(Self {
            data: Cachedata::new(),
            path,
        })
    }

    pub fn remove_cache(&self) -> Option<&Path> {
        if self.path.exists() {
            fs::remove_file(&self.path).ok()?;
            Some(self.path.as_path())
        } else {
            None
        }
    }

    pub fn new_with_path(path: PathBuf) -> Self {
        Self {
            data: Cachedata::new(),
            path,
        }
    }

    pub fn open_with_path(path: PathBuf, create: bool) -> Result<Self> {
        match (fs::read(&path), create) {
            (Ok(v), _) => Ok(Self {
                data: serde_json::from_slice(&v[..])
                    .map_err(|_| DestinyFetchError::CachePathErr)?,
                path,
            }),
            (Err(_), true) => Ok(Self::new_with_path(path)),
            _ => Err(DestinyFetchError::CachePathErr),
        }
    }

    pub fn open_default(create: bool) -> Result<Self> {
        let path = Self::default_path()?;
        match (fs::read(&path), create) {
            (Ok(v), _) => Ok(Self {
                data: serde_json::from_slice(&v[..])
                    .map_err(|_| DestinyFetchError::CachePathErr)?,
                path,
            }),
            (Err(_), true) => Ok(Self::new_with_path(path)),
            _ => Err(DestinyFetchError::CachePathErr),
        }
    }

    pub fn write_cache(&self) -> Result<()> {
        fs::write(&self.path, serde_json::to_vec_pretty(&self.data)?)?;
        Ok(())
    }
}

impl Cachedata {
    pub fn new_save_name(&self) -> String {
        format!("stored_query_{}", self.len() + 1)
    }
    pub fn new() -> Self {
        Cachedata {
            last_sync_at: None,
            last_sync_rows_written: None,
            cached_searches: HashMap::new(),
        }
    }
    pub fn open(path: PathBuf) -> Result<Cachedata> {
        let v: Vec<u8> = fs::read(path)?;
        serde_json::from_slice(&v[..]).map_err(|_| DestinyFetchError::CachePathErr)
    }
    pub fn len(&self) -> usize {
        self.cached_searches.len()
    }
    pub fn write(&self, path: PathBuf) -> Result<()> {
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    // pub fn write_images(&mut self, images: Vec<ImagesRow>, path: Option<PathBuf>) -> Result<()> {
    //     self.cached_searches.insert(self.new_save_name(), images);
    //     let path = path.unwrap_or(Self::default_path()?);
    //     fs::write(path, serde_json::to_string_pretty(self)?)?;
    //     Ok(())
    // }
    // pub fn write_images_with_name(
    //     &mut self,
    //     name: String,
    //     images: Vec<ImagesRow>,
    //     path: Option<PathBuf>,
    // ) -> Result<()> {
    //     self.cached_searches.insert(name, images);
    //     let path = path.unwrap_or(Self::default_path()?);
    //     fs::write(path, serde_json::to_string_pretty(self)?)?;
    //     Ok(())
    // }
    pub fn read_images(&self, name: impl AsRef<str>) -> Option<&Vec<ImagesRow>> {
        self.cached_searches.get(name.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_open_new_cache() {
        let failpath = PathBuf::from("failpath".to_string());
        let exp = Cache::new_with_path(failpath.clone());
        exp.remove_cache();
        let test = Cache::open_with_path(failpath, true).expect("failed to open Cache");
        assert_eq!(exp, test);
    }

    #[test]
    fn test_default_path() {
        let path = Cache::default_path().expect("unable to get cache path");
        dbg!(path);
    }

    #[test]
    fn test_store_images() {
        let mut cache = Cache::new().expect("failed to create new cache");
        let now: DateTime<Utc> = Local::now().to_utc();
        cache.data.last_sync_at = Some(now);
        cache.data.last_sync_rows_written = Some(10);
        let ser = serde_json::to_string_pretty(&cache.data).expect("unable to serialize cache");
        dbg!(ser);
    }
    #[test]
    fn test_store_images_null() {
        let cache = Cache::new().expect("failed to create new cache");
        let ser = serde_json::to_string_pretty(&cache.data).expect("unable to serialize cache");
        dbg!(ser);
    }
}
