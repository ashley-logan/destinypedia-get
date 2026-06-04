use super::{CACHE_FILE, Cache};
use crate::bin_modules::database::rows::ImagesRow;
use crate::bin_modules::{DestinyFetchError, Result};
use destinypedia::NAMESPACE::CATEGORY;
use dirs;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncReadExt};
use tokio::{fs, io};

async fn parse_cached_search(name: String) -> Result<Vec<ImagesRow>> {
    let mut cache_f = dirs::cache_dir().ok_or(DestinyFetchError::CachePathErr)?;
    cache_f.push(CACHE_FILE);
    let bytes = fs::read(&cache_f).await?;
    let v: Cache = serde_json::from_slice(&bytes[..])?;
    let images = v.cached_searches.get(&name);
    todo!("parse images or return error if None");
    Ok(vec![])
}

async fn parse_titles_file(fpath: PathBuf) -> Result<HashSet<String>> {
    let mut titles: HashSet<String> = HashSet::new();
    let rdr = io::BufReader::new(fs::File::open(&fpath).await?);
    let mut lines = rdr.lines();
    while let Some(title) = lines.next_line().await? {
        titles.insert(title.trim().into());
    }
    println!(
        "Successfully parsed {} titles from {} !",
        titles.len(),
        fpath.display()
    );
    Ok(titles)
}
async fn parse_ids_file(fpath: PathBuf) -> Result<HashSet<i32>> {
    let mut ids: HashSet<i32> = HashSet::new();
    let rdr = io::BufReader::new(fs::File::open(&fpath).await?);
    let mut lines = rdr.lines();
    while let Some(s) = lines.next_line().await? {
        ids.insert(s.parse().map_err(|_| {
            DestinyFetchError::ParseIdErr(s, fpath.to_str().unwrap_or("??").into())
        })?);
    }
    println!(
        "Successfully parsed {} ids from {} !",
        ids.len(),
        fpath.display()
    );

    Ok(ids)
}
