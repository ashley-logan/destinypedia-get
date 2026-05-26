use derive_more::{Display, Error, From};

#[derive(Debug, Error, From, Display)]
pub enum DatabaseError {
    ConnectionError(diesel::ConnectionError),
    #[display("Error converting response::Item type into database::Row type")]
    IntoRowConvertError,
}

pub type DatabaseResult<T> = std::result::Result<T, DatabaseError>;
