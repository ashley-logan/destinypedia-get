use crate::SubCategoryRow;
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

    PAGES
        id, name
    PAGE_CATEGORIES
        page_id, category_id
    PAGE_IMAGES
        page_id, image_id

    maybe: GRIMOIRE
*/

pub fn write_db_subcategories(
    conn: &Connection,
    recv: Receiver<SubCategoryRow>,
) -> crate::Result<()> {
    let mut tx = Transaction::new_unchecked(conn, rusqlite::TransactionBehavior::Deferred)?;
    let mut insert_count = 0_u32;

    let mut stmt = conn.prepare_cached(
        r"
        INSERT INTO SUBCATEGORIES (category_id, subcategory_id)
        VALUES (?1, ?2)",
    )?;
    while let Ok(row) = recv.recv() {
        stmt.execute((row.id, row.subcategory_id))?;
        insert_count += 1;

        if insert_count >= 300 {
            tx.commit()?;
            tx = Transaction::new_unchecked(conn, rusqlite::TransactionBehavior::Deferred)?;
            insert_count = 0;
        }
    }
    drop(stmt);
    tx.commit()?;

    Ok(())
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
                size INTEGER,
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
        let (sx, rx) = unbounded::<SubCategoryRow>();
        create_tables("data/dev.db".into());
        let conn = Connection::open("data/dev.db").unwrap();

        let mut ids: std::ops::Range<u32> = 0..1000;
        let mut subids: std::ops::Range<u32> = 200..1200;

        thread::scope(|s| {
            s.spawn(move || {
                write_db_subcategories(&conn, rx).unwrap();
            });
            s.spawn(move || {
                for _ in 0..1000 {
                    let (id, subid) = (ids.next().unwrap(), subids.next().unwrap());
                    sx.send(SubCategoryRow::from((id, subid))).unwrap();
                }
            });
        });
    }
}
