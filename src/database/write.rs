use crate::database::{CategoriesRow, ImageCategoryRow, ImagesRow, Row, SubCategoryRow};
use crossbeam_channel::{Receiver, unbounded};
use dirs;
use rusqlite::{Batch, Connection, Transaction};
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

pub fn dispatch_row_writer(
    conn: &Connection,
    recv: Receiver<Row>,
    batch_size: usize,
) -> crate::Result<()> {
    let (mut batch_counter, mut tx_insert_count, mut total_insert_count) =
        (0_usize, 0_usize, 0_usize);
    let mut tx = Transaction::new_unchecked(conn, rusqlite::TransactionBehavior::Deferred)?;
    while let Ok(row) = recv.recv() {
        let rows_inserted = match row {
            Row::SubCategory(sc) => write_row_subcategories(conn, sc),
            Row::ImageCategory(ic) => write_row_image_categories(conn, ic),
            Row::Categories(c) => write_row_categories(conn, c),
            Row::Images(i) => write_row_images(conn, i),
        }?;

        tx_insert_count += rows_inserted;

        if tx_insert_count >= batch_size {
            tx.commit()?;
            tx = Transaction::new_unchecked(conn, rusqlite::TransactionBehavior::Deferred)?;
            batch_counter += 1;
            dbg!(format!(
                "BATCH #{} WRITTEN => {} ROWS INSERTED",
                batch_counter, tx_insert_count
            ));
            total_insert_count += tx_insert_count;
            tx_insert_count = 0;
        }
    }
    tx.commit()?;
    batch_counter += 1;
    dbg!(format!(
        "FINAL BATCH #{} WRITTEN => {} ROWS INSERTED",
        batch_counter, tx_insert_count
    ));
    total_insert_count += tx_insert_count;
    dbg!(format!(
        "WRITER COMPLETE\n\tTOTAL ROWS INSERTED => {}\n\tTOTAL BATCHES PROCESSED => {}",
        total_insert_count, batch_counter
    ));
    Ok(())
}

pub fn write_row_subcategories(conn: &Connection, row: SubCategoryRow) -> crate::Result<usize> {
    let mut stmt = conn.prepare_cached(
        r"
        INSERT INTO SUBCATEGORIES (category_id, subcategory_id)
        VALUES (?1, ?2)",
    )?;

    stmt.execute((row.id, row.subcategory_id))
        .map_err(|e| e.into())
}

pub fn write_row_image_categories(
    conn: &Connection,
    row: ImageCategoryRow,
) -> crate::Result<usize> {
    let mut stmt = conn.prepare_cached(
        r"
        INSERT INTO IMAGE_CATEGORIES (image_id, category_id)
        VALUES (?1, ?2)",
    )?;
    stmt.execute((row.image_id, row.category_id))
        .map_err(|e| e.into())
}

pub fn write_row_images(conn: &Connection, row: ImagesRow) -> crate::Result<usize> {
    let mut stmt = conn.prepare_cached(
        r"
        INSERT INTO IMAGES (id, title, url, size, width, height, timestamp)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ",
    )?;

    const MIB: f64 = 1024.0 * 1024.0;
    let mib_size: f64 = (row.size as f64) / MIB;

    stmt.execute((
        row.id,
        row.title,
        row.url,
        mib_size,
        row.width,
        row.height,
        row.timestamp,
    ))
    .map_err(|e| e.into())
}

pub fn write_row_categories(conn: &Connection, row: CategoriesRow) -> crate::Result<usize> {
    let mut stmt = conn.prepare_cached(
        r"
        INSERT INTO CATEGORIES (id, title, subcats, files)
        VALUES (?1, ?2, ?3, ?4)
        ",
    )?;

    stmt.execute((row.id, row.title, row.subcats, row.files))
        .map_err(|e| e.into())
}

fn create_tables(conn_path: PathBuf) {
    // let ddir: PathBuf = dirs::data_dir().expect("ERROR: Couldn't find data directory");
    // let DB_URL = format!(
    //     "sqlite://{}",
    //     ddir.join("destinypedia.db").to_string_lossy()
    // );

    let mut conn = Connection::open(conn_path).unwrap();
    conn.pragma_update(None, "journal_mode", "WAL").unwrap();
    let tx = conn.transaction().unwrap();
    tx.execute(
        r"
            CREATE TABLE IF NOT EXISTS IMAGES (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                url TEXT,
                size REAL,
                width INTEGER,
                height INTEGER,
                timestamp TEXT

            )",
        (),
    )
    .unwrap();

    tx.execute(
        r"
            CREATE TABLE IF NOT EXISTS IMAGE_CATEGORIES (
                image_id INTEGER NOT NULL,
                category_id INTEGER NOT NULL,
                FOREIGN KEY (image_id) REFERENCES images(id),
                FOREIGN KEY (category_id) REFERENCES categories(id)

            )",
        (),
    )
    .unwrap();

    tx.execute(
        r"
            CREATE TABLE IF NOT EXISTS CATEGORIES (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                subcats INTEGER,
                files INTEGER
            )",
        (),
    )
    .unwrap();

    tx.execute(
        r#"
            CREATE TABLE IF NOT EXISTS SUBCATEGORIES (
                category_id INTEGER NOT NULL,
                subcategory_id INTEGER NOT NULL,
                FOREIGN KEY (category_id) REFERENCES categories(id),
                FOREIGN KEY (subcategory_id) REFERENCES categories(id)

            )"#,
        (),
    )
    .unwrap();

    tx.commit().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_table_creation() {
        create_tables(PathBuf::from("data/dev.db"));
    }

    #[test]
    fn test_writer_small() {
        let (sx, rx) = unbounded::<Row>();
        create_tables("data/dev.db".into());
        let conn = Connection::open("data/dev.db").unwrap();

        let mut ids: std::ops::Range<u32> = 0..1000;
        let mut subids: std::ops::Range<u32> = 200..1200;

        thread::scope(|s| {
            s.spawn(move || {
                dispatch_row_writer(&conn, rx, 300_usize).unwrap();
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
