mod cache;
pub mod cli;
pub mod database;
pub mod get;
pub mod sync;
pub use cache::Cache;
pub use database::schema::{categories, image_categories, images, subcategories};

#[derive(Debug, derive_more::Error, derive_more::From, derive_more::Display)]
#[from(forward)]
pub enum DestinyFetchError {
    #[from(
        serde_json::Error,
        reqwest::Error,
        destinypedia::request::error::RequestError
    )]
    RequestErr,
    #[from(destinypedia::response::error::ResponseError)]
    ResponseErr,
    #[from(
        rusqlite::Error,
        database::error::DatabaseError,
        diesel::ConnectionError,
        diesel::result::Error
    )]
    DatabaseErr,
    #[from(std::io::Error)]
    IOErr,
}

pub type Result<T> = std::result::Result<T, DestinyFetchError>;
