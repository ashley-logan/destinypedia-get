use derive_more::{Display, Error, From};

#[derive(Debug, Error, Display, From)]
pub enum RequestError {
    BuildQuery(serde_json::Error),
    ReqwestError(reqwest::Error),
}

pub type RequestResult<T> = std::result::Result<T, RequestError>;
