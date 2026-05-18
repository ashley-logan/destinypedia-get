use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum FromResultHelperError {
    #[display("failed to parse pageid (i32 could not be converted into u32)")]
    ConvertPageid,
    #[display("failed to parse namespace (u16 could not be converted into NAMESPACE)")]
    ConvertNamespace,
    #[display("result helper is missing necessary fields (pageid, title, ns)")]
    MissingField,
}

pub type ResponseResult<T> = std::result::Result<T, FromResultHelperError>;
