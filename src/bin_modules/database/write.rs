use super::rows::{CategoriesRow, ImageCategoryRow, ImagesRow, Row, SubCategoryRow};
use crossbeam_channel::{Receiver, unbounded};
use dirs;
use rusqlite::{Batch, Connection, Transaction};
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

pub fn dispatch_row_writer(
    conn: Connection,
    recv: Receiver<Row>,
    batch_size: usize,
) -> super::error::DatabaseResult<HashMap<String, usize>> {
    let mut counters: HashMap<String, usize> = HashMap::from_iter([
        ("batch_count".into(), 0_usize),
        ("insert_count".into(), 0_usize),
        ("total_insert_count".into(), 0_usize),
        ("subcat_count".into(), 0_usize),
        ("imgcat_count".into(), 0_usize),
        ("cat_count".into(), 0_usize),
        ("img_count".into(), 0_usize),
    ]);

    let mut tx = Transaction::new_unchecked(&conn, rusqlite::TransactionBehavior::Deferred)
        .expect("unable to open new transaction");
    while let Ok(row) = recv.recv() {
        let rows_inserted = match row {
            Row::SubCategory(sc) => {
                let n = write_row_subcategories(&conn, sc)?;
                counters
                    .entry("subcat_count".into())
                    .and_modify(|i| *i += n);
                n
            }
            Row::ImageCategory(ic) => {
                let n = write_row_image_categories(&conn, ic)?;
                counters
                    .entry("imgcat_count".into())
                    .and_modify(|i| *i += n);
                n
            }
            Row::Categories(c) => {
                let n = write_row_categories(&conn, c)?;
                counters.entry("cat_count".into()).and_modify(|i| *i += n);
                n
            }
            Row::Images(i) => {
                let n = write_row_images(&conn, i)?;
                counters.entry("img_count".into()).and_modify(|i| *i += n);
                n
            }
        };
        counters
            .entry("insert_count".into())
            .and_modify(|i| *i += rows_inserted);

        if counters["insert_count"] >= batch_size {
            tx.commit()?;
            tx = Transaction::new_unchecked(&conn, rusqlite::TransactionBehavior::Deferred)?;
            counters.entry("batch_count".into()).and_modify(|i| *i += 1);
            dbg!(format!(
                "BATCH #{} WRITTEN => {} ROWS INSERTED",
                counters["batch_count"], counters["insert_count"]
            ));

            *counters.get_mut("total_insert_count").unwrap() += counters["insert_count"];
            counters.insert("insert_count".into(), 0);
        }
    }
    tx.commit()?;
    counters.entry("batch_count".into()).and_modify(|i| *i += 1);
    dbg!(format!(
        "FINAL BATCH #{} WRITTEN => {} ROWS INSERTED",
        counters["batch_count"], counters["insert_count"]
    ));
    *counters.get_mut("total_insert_count").unwrap() += counters["insert_count"];
    counters.insert("insert_count".into(), 0);
    dbg!(format!(
        "WRITER COMPLETE\n\tTOTAL ROWS INSERTED => {}\n\tTOTAL BATCHES PROCESSED => {}",
        counters["total_insert_count"], counters["batch_count"]
    ));
    Ok(counters)
}

pub fn write_row_subcategories(
    conn: &Connection,
    row: SubCategoryRow,
) -> super::error::DatabaseResult<usize> {
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
) -> super::error::DatabaseResult<usize> {
    let mut stmt = conn.prepare_cached(
        r"
        INSERT INTO IMAGE_CATEGORIES (image_id, category_id)
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

pub fn create_tables(conn_path: impl AsRef<Path>) -> super::error::DatabaseResult<()> {
    // let ddir: PathBuf = dirs::data_dir().expect("ERROR: Couldn't find data directory");
    // let DB_URL = format!(
    //     "sqlite://{}",
    //     ddir.join("destinypedia.db").to_string_lossy()
    // );

    let mut conn: Connection = Connection::open(conn_path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    let tx = conn.transaction()?;
    tx.execute(
        r"
            CREATE TABLE IF NOT EXISTS IMAGES (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                size REAL,
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
        create_tables(PathBuf::from("data/dev.db"));
    }

    #[test]
    fn test_writer_small() {
        let (sx, rx) = unbounded::<Row>();
        create_tables(Path::new("data/dev.db"));
        let conn = Connection::open("data/dev.db").unwrap();

        let mut ids: std::ops::Range<u32> = 0..1000;
        let mut subids: std::ops::Range<u32> = 200..1200;

        thread::scope(|s| {
            s.spawn(move || {
                dispatch_row_writer(conn, rx, 300_usize).unwrap();
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
