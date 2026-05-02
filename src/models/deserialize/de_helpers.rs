use crate::models::deserialize::prop_results::PropResults;
use crate::models::deserialize::response::QueryResult;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, PartialEq, Eq)]
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
