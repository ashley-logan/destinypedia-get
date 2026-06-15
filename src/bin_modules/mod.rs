mod cache;
pub mod cli;
pub mod database;
pub mod get;
pub mod sync;
pub use cache::{CACHE_FILE, Cache};
mod logging;
pub use logging::setup_logging;
pub mod download;
pub mod input;
pub mod interactive;
pub mod search;

#[derive(Debug, derive_more::Error, derive_more::From, derive_more::Display)]
#[from(forward)]
pub enum DestinyFetchError {
    #[display("User quit program")]
    Quit,
    #[display("ERROR: Unable to run program, unknown error occurred")]
    Unknown,
    #[display("ERROR: No matching images found")]
    NoMatchingImages,
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
    #[from]
    InteractiveErr(#[from] inquire::InquireError),
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
    #[from]
    IOErr(#[from] std::io::Error),
    #[from(skip)]
    #[display("no integer argument can be negative")]
    NegativeArgErr,
    #[from(skip)]
    #[display("ERROR: While parsing input file {_0}, failed to parse {_1} as i32.")]
    ParseIdErr(String, String),
    #[from(skip)]
    #[display("attempted to call categories/images method on incorrect result_type")]
    WrongQueryMethod,
    #[from(skip)]
    #[display("invalid path; path does not exist")]
    InvalidPathErr,
    #[from(skip)]
    #[display("Unable to parse Ext from str")]
    ExtFromStrErr,
    #[from(skip)]
    #[display("ERROR: No cache found")]
    CachePathErr,
    #[from(skip)]
    #[display("ERROR: No cached data matching name found")]
    NotCachedErr,
    #[from]
    InvalidTimestampErr(#[from] chrono::ParseError),
    #[from(skip)]
    #[display("missing required argument")]
    MissingArgErr,
}

pub enum UserError {
    InvalidTitles(std::collections::HashSet<String>),
    InvalidIds(std::collections::HashSet<i32>),
    InvalidCategory(String),
}
impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTitles(v) => {
                writeln!(f, "Invalid titles: ");
                v.iter().for_each(|i| {
                    writeln!(f, "\t{}", i);
                });
            }
            Self::InvalidIds(v) => {
                writeln!(f, "Invalid ids: ");
                v.iter().for_each(|i| {
                    writeln!(f, "\t{}", i);
                });
            }
            Self::InvalidCategory(c) => {
                writeln!(f, "Invalid category: {}", c);
            }
        }
        Ok(())
    }
}

pub type Result<T> = std::result::Result<T, DestinyFetchError>;
