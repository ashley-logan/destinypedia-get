use crate::models::deserialize::prop_results::*;
use crate::models::deserialize::query::QueryResult;
use serde::Deserialize;
use serde_with::{DefaultOnError, TryFromInto};
use std::collections::HashMap;

#[serde_with::serde_as]
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct QueryResultHelper {
    pub pageid: Option<i32>,
    #[serde_as(as = "DefaultOnError<Option<TryFromInto<u16>>>")]
    pub ns: Option<crate::models::NAMESPACE>,
    pub title: Option<String>,
    pub categories: Option<CategoriesProp>,
    pub categoryinfo: Option<CategoryInfoProp>,
    pub images: Option<ImagesProp>,
    pub imageinfo: Option<ImageInfoProp>,
    #[serde(flatten)]
    pub pageimages: Option<PageImagesProp>,
    #[serde(flatten)]
    pub info: Option<InfoProp>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct IndiscriminateQueryHelper {
    pub(crate) pageids: Option<Vec<u32>>,
    pub(crate) pages: HashMap<String, QueryResult>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct QueryResponseHelper {
    pub(crate) query: IndiscriminateQueryHelper,
}
