use crate::models::deserialize::prop_results::*;
use crate::models::deserialize::query::{Continue, IndiscriminateQueryResult, Query, QueryResult};
use serde::{Deserialize, de::DeserializeOwned};
use std::collections::HashMap;
use serde_with::DeserializeAs;


#[derive(Debug, Deserialize)]
pub(crate) struct QueryResultHelper<T: PropResults> {
    pub(crate) pageid: Option<usize>,
    pub(crate) ns: Option<usize>,
    pub(crate) title: Option<String>,
    pub(crate) missing: Option<String>,
    #[serde(flatten)]
    pub(crate) items: Option<T>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct QueryHelper<T: PropResults> {
    pub(crate) pages: Option<HashMap<String, QueryResult<T>>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ResponseHelper<T: PropResults> {
    #[serde(rename = "continue")]
    pub(crate) cont: Option<Continue>,
    pub(crate) query: Option<Query<T>>,
}


/* 
use serde_with::DeserializeAs;

pub struct PointFromRaw;

impl<'de> DeserializeAs<'de, Point> for PointFromRaw {
    fn deserialize_as<D>(deserializer: D) -> Result<Point, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Step 1: Deserialize into the helper
        let raw = PointRaw::deserialize(deserializer)?;

        // Step 2: Convert helper → real type
        Ok(Point {
            x: raw.x,
            y: raw.y,
        })
    }
}

    */
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct IndiscriminateQueryResultHelper {
    pub pageid: Option<usize>,
    pub ns: Option<usize>,
    pub title: Option<String>,
    pub missing: Option<String>,
    #[serde(flatten)]
    pub categories: Option<Categories>,
    #[serde(flatten)]
    pub categoryinfo: Option<CategoryInfo>,
    #[serde(flatten)]
    pub images: Option<Images>,
    #[serde(flatten)]
    pub pageimages: Option<PageImages>,
    #[serde(flatten)]
    pub imageinfo: Option<ImageInfo>,
    #[serde(flatten)]
    pub info: Option<PageInfo>,
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
