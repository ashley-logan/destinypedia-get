mod cache;
pub mod cli;
pub mod database;
pub mod get;
pub mod sync;
pub use cache::Cache;
pub mod search;
pub use database::schema::{categories, image_categories, images, subcategories};
pub use database::tables::{Categories, ImageCategories, Images, Subcategories};

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
        database::error::DatabaseError,
        diesel::ConnectionError,
        diesel::result::Error
    )]
    DatabaseErr,
    #[from(std::io::Error, csv::Error)]
    IOErr,
    #[from(skip)]
    #[display("no integer argument can be negative")]
    NegativeArgErr,
    #[from(skip)]
    #[display("attempted to call categories/images method on incorrect result_type")]
    WrongQueryMethod,
    #[from(skip)]
    #[display("invalid path; path does not exist")]
    InvalidPathErr,
    #[from(chrono::ParseError)]
    InvalidTimestampErr,
    #[from(skip)]
    #[display("missing required argument")]
    MissingArgErr,
}

pub type Result<T> = std::result::Result<T, DestinyFetchError>;
