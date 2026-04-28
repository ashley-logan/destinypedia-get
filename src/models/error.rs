use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error parsing cli arguments")]
    Args,
}

pub type Result<T> = std::result::Result<T, self::Error>;
