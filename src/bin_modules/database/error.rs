use derive_more::{Display, Error, From};

#[derive(Debug, Error, From, Display)]
pub enum DatabaseError {
    ConnectionError(rusqlite::Error),
    #[display("Error converting response::Item type into database::Row type")]
    IntoRowConvertError,
}

pub type DatabaseResult<T> = rusqlite::Result<T, DatabaseError>;
