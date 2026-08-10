use super::{CACHE_FILE, Cache};
use crate::bin_modules::database::rows::ImagesRow;
use crate::bin_modules::{DestinyFetchError, Result};
use dirs;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncReadExt};
use tokio::{fs, io};

// pub async fn parse_cached_search(name: String) -> Result<Vec<ImagesRow>> {
//     let mut cache_f = dirs::cache_dir().ok_or(DestinyFetchError::CachePathErr)?;
//     cache_f.push(CACHE_FILE);
//     let bytes = fs::read(&cache_f).await?;
//     let v: Cache = serde_json::from_slice(&bytes[..])?;
//     let images: Option<&Vec<ImagesRow>> = v.cached_searches.get(&name);
//     match images {
//         Some(v) => Ok(v.to_vec()),
//         None => Err(DestinyFetchError::NotCachedErr),
//     }
// }

pub fn titles_from_file(path: PathBuf) -> Result<Vec<String>> {
    let mut titles: Vec<String> = vec![];
    let f = std::fs::File::open(&path)?;
    let rdr = std::io::BufReader::new(f);
    for ln in rdr.lines() {
        let title: String = ln?.trim().to_string();
        titles.push(title);
    }
    println!(
        "Successfully parsed {} titles from {} !",
        titles.len(),
        path.display()
    );
    Ok(titles)
}

pub fn ids_from_file(path: PathBuf) -> Result<Vec<u16>> {
    let mut ids: Vec<u16> = vec![];
    let f = std::fs::File::open(&path)?;
    let rdr = std::io::BufReader::new(f);
    for ln in rdr.lines() {
        let id: u16 = ln?.parse().map_err(|_| DestinyFetchError::NegativeArgErr)?;
        ids.push(id);
    }
    println!(
        "Successfully parsed {} titles from {} !",
        ids.len(),
        path.display()
    );
    Ok(ids)
}

pub async fn parse_titles_file(fpath: PathBuf) -> Result<Vec<String>> {
    let mut titles: Vec<String> = vec![];
    let rdr = io::BufReader::new(fs::File::open(&fpath).await?);
    let mut lines = rdr.lines();
    while let Some(title) = lines.next_line().await? {
        titles.push(title.trim().into());
    }
    println!(
        "Successfully parsed {} titles from {} !",
        titles.len(),
        fpath.display()
    );
    Ok(titles)
}
pub async fn parse_ids_file(fpath: PathBuf) -> Result<Vec<i32>> {
    let mut ids: Vec<i32> = vec![];
    let rdr = io::BufReader::new(fs::File::open(&fpath).await?);
    let mut lines = rdr.lines();
    while let Some(s) = lines.next_line().await? {
        ids.push(s.parse().map_err(|_| {
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Connection, sqlite::SqliteConnection};

    const TEST_DB: &str = "data/dev.db";

    #[sqlx::test]
    async fn test_parse_cache() {
        let mut conn = SqliteConnection::connect(TEST_DB)
            .await
            .expect("failed to connect to the test database");
        let images: Vec<ImagesRow> = sqlx::query_as!(
            ImagesRow,
            r"select * from images
            order by random()
            limit 15"
        )
        .fetch_all(&mut conn)
        .await
        .expect("failed to query random rows from test database");
    }
}
