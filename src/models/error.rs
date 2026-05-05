use reqwest;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error parsing cli arguments")]
    Args,
    #[error("Error sending or parsing request")]
    RequestErr(#[from] reqwest::Error),
    #[error("Error constructing PARAMS object")]
    Params,
}

pub type Result<T> = std::result::Result<T, self::Error>;
