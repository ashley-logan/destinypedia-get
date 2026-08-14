use super::database::rows::{CategoriesRow, ImagesRow};
use super::{DestinyFetchError, Result};
use chrono::Utc;
use dirs::data_dir;
use serde_json::to_writer;
use std::path::{Path, PathBuf};
use std::{fs, io};

const STORE_DIR: &'static str = "destiny_fetch/saved-searches";

pub fn store_images(images: &[ImagesRow], name: &Option<String>) -> Result<PathBuf> {
    let name: String = name.clone().unwrap_or(Utc::now().to_string());
    let mut path = data_dir().ok_or(DestinyFetchError::AppDataPathErr)?;
    path.push(Path::new(STORE_DIR));
    fs::create_dir_all(&path)?;
    path.push(Path::new(&name));
    let f: fs::File = fs::File::create(&path)?;
    to_writer(f, images)?;
    Ok(path)
}
