mod cache;
pub mod cli;
pub mod database;
pub mod get;
pub mod sync;
pub use cache::Cache;
mod logging;
pub use logging::setup_logging;
pub mod search;

#[derive(Debug, derive_more::Error, derive_more::From, derive_more::Display)]
#[from(forward)]
pub enum DestinyFetchError {
    #[from]
    SyncErr(#[from] tokio::task::JoinError),
    #[from]
    DatabaseMigrateErr(#[from] sqlx::migrate::MigrateError),
    #[from(
        serde_json::Error,
        reqwest::Error,
        destinypedia::request::error::RequestError
    )]
    RequestErr,
    #[from(destinypedia::response::error::ResponseError)]
    ResponseErr,
    #[from]
    SqlxErr(#[from] sqlx::Error),
    #[from]
    RecvTimeoutErr(#[from] crossbeam_channel::RecvTimeoutError),
    #[from]
    AsyncRecvErr(#[from] async_channel::RecvError),
    #[from]
    RowError(#[from] super::database::error::DatabaseError),
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
    #[from]
    InvalidTimestampErr(#[from] chrono::ParseError),
    #[from(skip)]
    #[display("missing required argument")]
    MissingArgErr,
}

pub type Result<T> = std::result::Result<T, DestinyFetchError>;
