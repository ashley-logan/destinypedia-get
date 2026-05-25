use super::rows::{CategoriesRow, ImageCategoryRow, ImagesRow, Row, SubCategoryRow};
use super::schema::{categories, image_categories, images, subcategories};
use crossbeam_channel::{Receiver, unbounded};
use diesel::insert_or_ignore_into;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dirs;
use rusqlite::{Batch, Transaction};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
/*
DATABASE SCHEMA
    IMAGES
        id, name, size, width, height, url, timestamp
    IMAGE_CATEGORIES
        image_id, category_id
    CATEGORIES
        id, name, size,

    SUBCATEGORIES
        category_id, subcategory_id

    maybe: GRIMOIRE
*/

pub fn write_row_subcategories(
    conn: &SqliteConnection,
    row: SubCategoryRow<'_>,
) -> super::error::DatabaseResult<usize> {
    use subcategories::dsl::*;
    let mut stmt= insert_or_ignore_into(subcategories).values(row);
}

pub fn write_row_image_categories(
    conn: &Connection,
    row: ImageCategoryRow,
) -> super::error::DatabaseResult<usize> {
    let mut stmt = conn.prepare_cached(
        r"
        INSERT OR IGNORE INTO IMAGE_CATEGORIES (image_id, category_id)
        VALUES (?1, ?2)",
    )?;
    stmt.execute((row.image_id, row.category_id))
        .map_err(|e| e.into())
}

pub fn write_row_images(conn: &Connection, row: ImagesRow) -> super::error::DatabaseResult<usize> {
    let mut stmt = conn.prepare_cached(
        r"
        INSERT OR IGNORE INTO IMAGES (id, title, url, size, width, height, timestamp)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ",
    )?;

    stmt.execute((
        row.id,
        row.title,
        row.url,
        row.size,
        row.width,
        row.height,
        row.timestamp,
    ))
    .map_err(|e| e.into())
}

pub fn write_row_categories(
    conn: &Connection,
    row: CategoriesRow,
) -> super::error::DatabaseResult<usize> {
    let mut stmt = conn.prepare_cached(
        r"
        INSERT OR IGNORE INTO CATEGORIES (id, title, subcats, files)
        VALUES (?1, ?2, ?3, ?4)
        ",
    )?;

    stmt.execute((row.id, row.title, row.subcats, row.files))
        .map_err(|e| e.into())
}

pub fn create_tables(conn: &Connection) -> super::error::DatabaseResult<()> {
    // let ddir: PathBuf = dirs::data_dir().expect("ERROR: Couldn't find data directory");
    // let DB_URL = format!(
    //     "sqlite://{}",
    //     ddir.join("destinypedia.db").to_string_lossy()
    // );

    conn.pragma_update(None, "journal_mode", "WAL")?;
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        r"
            CREATE TABLE IF NOT EXISTS IMAGES (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                size INTEGER,
                width INTEGER,
                height INTEGER,
                timestamp TEXT

            )",
        (),
    )?;

    tx.execute(
        r"
            CREATE TABLE IF NOT EXISTS IMAGE_CATEGORIES (
                image_id INTEGER NOT NULL,
                category_id INTEGER NOT NULL,
                PRIMARY KEY (image_id, category_id),
                FOREIGN KEY (image_id) REFERENCES images(id),
                FOREIGN KEY (category_id) REFERENCES categories(id)

            )",
        (),
    )?;

    tx.execute(
        r"
            CREATE TABLE IF NOT EXISTS CATEGORIES (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                subcats INTEGER,
                files INTEGER
            )",
        (),
    )?;

    tx.execute(
        r#"
            CREATE TABLE IF NOT EXISTS SUBCATEGORIES (
                category_id INTEGER NOT NULL,
                subcategory_id INTEGER NOT NULL,
                PRIMARY KEY (category_id, subcategory_id),
                FOREIGN KEY (category_id) REFERENCES categories(id),
                FOREIGN KEY (subcategory_id) REFERENCES categories(id)

            )"#,
        (),
    )?;

    tx.commit().map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_table_creation() {
        let mut conn = Connection::open("data/dev.db").unwrap();
        create_tables(&mut conn);
    }

    #[test]
    fn test_writer_small() {
        let (sx, rx) = unbounded::<Row>();
        let mut conn = Connection::open("data/dev.db").unwrap();
        create_tables(&mut conn);

        let mut ids: std::ops::Range<u32> = 0..1000;
        let mut subids: std::ops::Range<u32> = 200..1200;

        thread::scope(|s| {
            s.spawn(move || {
                crate::bin_modules::sync::dispatch_row_writer(conn, rx, 300_usize).unwrap();
            });
            s.spawn(move || {
                for _ in 0..1000 {
                    let (id, subid) = (ids.next().unwrap(), subids.next().unwrap());
                    sx.send(Row::SubCategory(SubCategoryRow::from((id, subid))))
                        .unwrap();
                }
            });
        });
    }
}
