use crate::response::items::*;
use serde::Deserialize;
use std::collections::HashMap;

#[serde_with::serde_as]
#[derive(Debug, Deserialize)]
pub(super) struct QueryResultHelper {
    pub pageid: Option<i32>,
    pub(super) ns: Option<u16>,
    pub(super) title: Option<String>,
    pub(super) categories: Option<CategoriesProp>,
    pub(super) categoryinfo: Option<CategoryInfoProp>,
    pub(super) images: Option<ImagesProp>,
    pub(super) imageinfo: Option<ImageInfoProp>,
    #[serde(flatten)]
    pub(super) pageimages: Option<PageImagesProp>,
    #[serde(flatten)]
    pub(super) info: Option<InfoProp>,
}

#[derive(Debug, Deserialize)]
pub(super) struct IndiscriminateQueryHelper {
    pub(super) pageids: Option<Vec<i32>>,
    pub(super) pages: HashMap<String, QueryResultHelper>,
}

#[derive(Debug, Deserialize)]
pub(super) struct QueryResponseHelper {
    pub(super) query: IndiscriminateQueryHelper,
}
