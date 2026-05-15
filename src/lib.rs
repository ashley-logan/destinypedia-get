mod cli;
mod database;
mod get;
mod models;
mod prebuilt;
mod sync;
pub use cli::CLI;
pub use database::Row;
pub use models::deserialize;
pub use models::error::{self, Error, Result};
pub use models::serialize;
pub use models::{Continue, PARAMS, ParamsBuilder, Prop, QueryResponse};
pub use prebuilt::{category_members, image_info, images, page_images};
/*
action=query&list=search&srsearch=zavala&srnamespace=100&srlimit=max&srwhat=txt&srprop=titlesnippet|sectionsnippet|sectiontitle|size|timestamp&srsort=just_match&format=jsonfm
action=opensearch&search=hive&namespace=0&limit=max&profile=normal-subphrases&format=jsonfm
action=query&list=categorymembers&cmtitle=Category:Grimoire&cmprop=title|ids|sortkeyprefix|type&cmlimit=max&format=jsonfm
*/
