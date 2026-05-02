use crate::models::deserialize::prop_results;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Response<T: prop_results::PropResults> {
    #[serde(rename = "continue")]
    cont: Option<Continue>,
    query: Option<Query<T>>,
}
#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Continue {
    #[serde(rename = "continue")]
    contin: String,
    #[serde(flatten)]
    sub_cont: HashMap<String, String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Query<T: prop_results::PropResults> {
    // will usually deserialize from 'pageid': [items] or 'pageid': {item_fields}
    pages: Option<HashMap<String, ResultBase<T>>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct ResultBase<T: prop_results::PropResults> {
    pageid: u32,
    ns: u32,
    title: String,
    #[serde(flatten)]
    items: Option<HashMap<String, T>>,
}
