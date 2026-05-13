pub mod params;
pub mod query;
pub use params::{Format, PARAMS, ParamsBuilder};
pub use query::{GcmIdentifier, Generator, Query};
pub mod properties;
pub use properties::Prop;
pub mod ser_types;
pub use ser_types::Limit;

use serde::{Deserialize, Serialize};
use std::fmt;

impl fmt::Display for NAMESPACE {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u16)
    }
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

#[repr(u16)]
#[derive(
    Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, derive_more::TryFrom,
)]
#[try_from(repr)]
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
