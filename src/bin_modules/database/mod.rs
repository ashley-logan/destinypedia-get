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

pub async fn database_size_equals(expected_size: i64, conn: sqlx::sqlite::SqlitePool) -> bool {
    let mut db_size: i64 = 0;

    match sqlx::query_scalar!("SELECT COUNT(*) FROM images")
        .fetch_one(&conn)
        .await
    {
        Ok(count) => db_size += count,
        Err(_) => return false,
    }

    match sqlx::query_scalar!("SELECT COUNT(*) FROM categories")
        .fetch_one(&conn)
        .await
    {
        Ok(count) => db_size += count,
        Err(_) => return false,
    }

    match sqlx::query_scalar!("SELECT COUNT(*) FROM subcategories")
        .fetch_one(&conn)
        .await
    {
        Ok(count) => db_size += count,
        Err(_) => return false,
    }

    match sqlx::query_scalar!("SELECT COUNT(*) FROM image_categories")
        .fetch_one(&conn)
        .await
    {
        Ok(count) => db_size += count,
        Err(_) => return false,
    }

    db_size == expected_size
}
