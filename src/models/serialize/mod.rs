pub mod params;
pub mod query;
pub use params::PARAMS;
pub mod ser_types;

use serde::Serialize;
use std::fmt;

impl fmt::Display for NAMESPACE {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

#[repr(u8)]
#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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
