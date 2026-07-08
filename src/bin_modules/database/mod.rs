pub mod error;
pub mod rows;
// pub mod schema;
// pub mod tables;
// mod write;

pub fn get_db_path() -> super::Result<std::path::PathBuf> {
    Ok(dirs::data_local_dir()
        .or(dirs::data_dir())
        .ok_or(super::DestinyFetchError::InvalidPathErr)?
        .join("destiny_fetch.db"))
}
