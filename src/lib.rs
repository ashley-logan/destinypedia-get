mod cli;
mod database;
// mod get;
mod models;
mod prebuilt;
pub use cli::CLI;
pub use models::deserialize;
pub use models::error::{Error, Result};
pub use models::parse;
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

/*
action=query&list=search&srsearch=zavala&srnamespace=100&srlimit=max&srwhat=txt&srprop=titlesnippet|sectionsnippet|sectiontitle|size|timestamp&srsort=just_match&format=jsonfm
action=opensearch&search=hive&namespace=0&limit=max&profile=normal-subphrases&format=jsonfm
action=query&list=categorymembers&cmtitle=Category:Grimoire&cmprop=title|ids|sortkeyprefix|type&cmlimit=max&format=jsonfm
*/
