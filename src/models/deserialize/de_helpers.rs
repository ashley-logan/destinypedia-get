use crate::models::deserialize::prop_results::*;
use crate::models::deserialize::query::{Continue, IndiscriminateQueryResult};
use serde::{Deserialize, de::DeserializeOwned};
use serde_with::DeserializeAs;
use serde_with::{DefaultOnError, TryFromInto};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub(crate) struct QueryResultHelper<T: PropResults> {
    pub(crate) pageid: Option<usize>,
    pub(crate) ns: Option<usize>,
    pub(crate) title: Option<String>,
    pub(crate) missing: Option<String>,
    #[serde(flatten)]
    pub(crate) items: Option<T>,
}

// #[derive(Debug, Deserialize)]
// pub(crate) struct QueryHelper<T: PropResults> {
//     pub(crate) pages: Option<HashMap<String, QueryResult<T>>>,
// }

// #[derive(Debug, Deserialize)]
// pub(crate) struct ResponseHelper<T: PropResults> {
//     #[serde(rename = "continue")]
//     pub(crate) cont: Option<Continue>,
//     pub(crate) query: Option<Query<T>>,
// }



#[serde_with::serde_as]
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct IndiscriminateQueryResultHelper {
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
    pub(crate) pages: HashMap<String, IndiscriminateQueryResult>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct IndiscriminateResponseHelper {
    #[serde(rename = "continue")]
    pub(crate) cont: Option<Continue>,
    pub(crate) query: IndiscriminateQueryHelper,
}
