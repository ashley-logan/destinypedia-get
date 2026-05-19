use crate::response::items::*;
use serde::Deserialize;
use std::collections::HashMap;

#[serde_with::serde_as]
#[derive(Debug, Deserialize)]
pub(super) struct QueryResultHelper {
    pub pageid: Option<i32>,
    pub(super) ns: Option<crate::NAMESPACE>,
    pub(super) title: Option<String>,
    pub(super) categories: Option<Categories>,
    pub(super) categoryinfo: Option<CategoryInfo>,
    pub(super) images: Option<Images>,
    pub(super) imageinfo: Option<ImageInfo>,
    #[serde(flatten)]
    pub(super) pageimages: Option<PageImages>,
    #[serde(flatten)]
    pub(super) info: Option<Info>,
}

#[derive(Debug, Deserialize)]
pub(super) struct IndiscriminateQueryHelper {
    pub(super) pageids: Option<Vec<i32>>,
    pub(super) pages: HashMap<String, QueryResultHelper>,
}

#[derive(Debug, Deserialize)]
pub(super) struct QueryResponseHelper {
    #[serde(rename = "continue")]
    pub(super) continue_: Option<super::Continue>,
    pub(super) query: IndiscriminateQueryHelper,
}
