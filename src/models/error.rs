use crossbeam_channel::{RecvError, SendError};
use reqwest;
use serde_json;
use thiserror::Error;
use rusqlite::Error as RusqlError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error parsing cli arguments")]
    Args,
    #[error("Error sending or parsing request")]
    RequestErr(#[from] reqwest::Error),
    #[error("Error constructing PARAMS object")]
    Params,
    #[error("{0}")]
    SerdeJsonErr(#[from] serde_json::Error),
    #[error("Error converting into QueryResult")]
    TryIntoQueryResult,
    #[error("Error converting a response result into a row struct")]
    TryFromResponseIntoRow,
    #[error("{0}")]
    ChannelSendErr(#[from] SendError<Vec<u8>>),
    #[error("{0}")]
    ChannelRecieveErr(#[from] RecvError),
    #[error("{0}")]
    DatabaseError(#[from] RusqlError)
}

pub type Result<T> = std::result::Result<T, self::Error>;
