mod cli;
mod get;
mod models;
mod prebuilt;
pub use cli::CLI;
pub use models::error::{Error, Result};
pub use models::params::{Action, ErrorFormat, Format, PARAMS};
pub use models::parse;
pub use models::query;
pub use models::traits::ActionType;
pub use prebuilt::{category_members, image_info, images, page_images};

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
