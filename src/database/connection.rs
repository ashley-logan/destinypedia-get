use dirs;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
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

#[tokio::main]
async fn main() {
    let ddir: PathBuf = dirs::data_dir().expect("ERROR: Couldn't find data directory");
    let DB_URL = format!(
        "sqlite://{}",
        ddir.join("destinypedia.db").to_string_lossy()
    );

    let pool: SqlitePool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&DB_URL)
        .await
        .expect("Failed to connect to database");

    sqlx::query!(
        r"
            CREATE TABLE IF NOT EXISTS images (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                url TEXT,
                size INTEGER,
                width INTEGER,
                height INTEGER,
                timestamp TEXT

            )
        "
    )
    .execute(&pool)
    .await
    .expect("Failed to crate table images");

    sqlx::query!(
        r"
            CREATE TABLE IF NOT EXISTS image_categories (
                image_id INTEGER NOT NULL,
                category_id INTEGER NOT NULL,
                FOREIGN KEY (image_id) REFERENCES images(id),
                FOREIGN KEY (category_id) REFERENCES categories(id)

            )
        "
    )
    .execute(&pool)
    .await
    .expect("Failed to crate table image_categories");

    sqlx::query!(
        r"
            CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                size INTEGER
            )
        "
    )
    .execute(&pool)
    .await
    .expect("Failed to crate table categories");

    sqlx::query!(
        r#"
            CREATE TABLE IF NOT EXISTS subcategories (
                category_id INTEGER NOT NULL,
                subcategory_id INTEGER NOT NULL,
                FOREIGN KEY (category_id) REFERENCES categories(id),
                FOREIGN KEY (subcategory_id) REFERENCES categories(id)

            )
        "#
    )
    .execute(&pool)
    .await
    .expect("Failed to crate table subcategories");

    sqlx::query!(
        r"
            CREATE TABLE IF NOT EXISTS pages (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )
        "
    )
    .execute(&pool)
    .await
    .expect("Failed to crate table pages");

    sqlx::query!(
        r"
            CREATE TABLE IF NOT EXISTS page_categories (
                page_id INTEGER NOT NULL,
                category_id INTEGER NOT NULL,
                FOREIGN KEY (page_id) REFERENCES page(id),
                FOREIGN KEY (category_id) REFERENCES categories(id)
            )
        "
    )
    .execute(&pool)
    .await
    .expect("Failed to crate table page_categories");

    sqlx::query!(
        r"
            CREATE TABLE IF NOT EXISTS page_images (
                page_id INTEGER NOT NULL,
                image_id INTEGER NOT NULL,
                FOREIGN KEY (page_id) REFERENCES page(id),
                FOREIGN KEY (image_id) REFERENCES images(id)

            )
        "
    )
    .execute(&pool)
    .await
    .expect("Failed to crate table page_images");
}
