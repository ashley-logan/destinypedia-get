use dirs;
use rusqlite::{Connection};
use std::path::{PathBuf, Path};
use std::env;
use crate::SubCategoryRow;
use crossbeam_channel::Receiver;
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

pub fn write_db_subcategories(conn: &mut Connection, recv: Receiver<SubCategoryRow>) -> crate::Result<()> {
    let mut tx = conn.transaction()?;
    let mut insert_count = 0_u32;

    let mut stmt = tx.prepare_cached(
        r"
        INSERT INTO SUBCATEGORIES (category_id, subcategory_id)
        VALUES (?1, ?2)"
    )?;

    while let Ok(row) = recv.recv() {
        stmt.execute((row.id, row.subcategory_id))?;
        insert_count += 1;

        if insert_count >= 300 {
            tx.commit()?;
            tx = conn.transaction()?;
            stmt = tx.prepare_cached(
                r"
                INSERT INTO SUBCATEGORIES (category_id, subcategory_id)
                VALUES (?1, ?2)"
                )?;
            insert_count = 0;
        }

    }

    if insert_count > 0 {
        tx.commit()?;
    }

    Ok(())

    
}

fn create_tables() {
    // let ddir: PathBuf = dirs::data_dir().expect("ERROR: Couldn't find data directory");
    // let DB_URL = format!(
    //     "sqlite://{}",
    //     ddir.join("destinypedia.db").to_string_lossy()
    // );

    let conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "journal_mode", "WAL").unwrap();


    conn.execute(
        r"
            CREATE TABLE IF NOT EXISTS IMAGES (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                url TEXT,
                size INTEGER,
                width INTEGER,
                height INTEGER,
                timestamp TEXT

            )", ()
    ).unwrap();

    conn.execute(
        r"
            CREATE TABLE IF NOT EXISTS IMAGE_CATEGORIES (
                image_id INTEGER NOT NULL,
                category_id INTEGER NOT NULL,
                FOREIGN KEY (image_id) REFERENCES images(id),
                FOREIGN KEY (category_id) REFERENCES categories(id)

            )", ()
    ).unwrap();

    conn.execute(
        r"
            CREATE TABLE IF NOT EXISTS CATEGORIES (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                subcats INTEGER,
                files INTEGER
            )", ()
    ).unwrap();

     conn.execute(
        r#"
            CREATE TABLE IF NOT EXISTS SUBCATEGORIES (
                category_id INTEGER NOT NULL,
                subcategory_id INTEGER NOT NULL,
                FOREIGN KEY (category_id) REFERENCES categories(id),
                FOREIGN KEY (subcategory_id) REFERENCES categories(id)

            )"#, ()
    ).unwrap();
}
