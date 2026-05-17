mod cli;
mod database;
pub mod get;
pub mod request;
pub mod response;
pub mod sync;
pub use cli::CLI;
pub use database::Row;

#[repr(u16)]
#[derive(
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    derive_more::TryFrom,
)]
#[try_from(repr)] // u16.try_into() --> Result<NAMESPACE>
pub enum NAMESPACE {
    PAGE = 0,
    TALK = 1,
    USER = 2,
    USERTALK = 3,
    DESTINYPEDIA = 4,
    DESTINYPEDIATALK = 5,
    FILE = 6,
    FILETALK = 7,
    MEDIAWIKI = 8,
    TEMPLATE = 10,
    TEMPLATETALK = 11,
    HELP = 12,
    HELPTALK = 13,
    CATEGORY = 14,
    GRIMOIRE = 100,
    FORUM = 110,
}

impl Into<String> for NAMESPACE {
    fn into(self) -> String {
        (self as u16).to_string()
    }
}

impl Into<u16> for NAMESPACE {
    fn into(self) -> u16 {
        self as u16
    }
}
