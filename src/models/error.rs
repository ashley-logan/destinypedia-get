use reqwest;
use serde_json;
use thiserror::Error;

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
}

pub type Result<T> = std::result::Result<T, self::Error>;
