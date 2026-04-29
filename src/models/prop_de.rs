// use serde::Deserialize;
// use std::collections::HashMap;

// #[derive(Debug, Deserialize, PartialEq, Eq)]
// pub struct PropResponse {
//     #[serde(rename = "continue")]
//     cont: Option<Continue>,
//     query: Query,
// }
// #[derive(Debug, Deserialize, PartialEq, Eq)]
// struct Continue {
//     #[serde(rename = "continue")]
//     contin: String,
//     #[serde(flatten)]
//     sub_cont: SubContinue,
// }

// #[derive(Debug, Deserialize, PartialEq, Eq)]
// struct Query {
//     pages: Option<HashMap<String, Page>>,
// }

// #[derive(Debug, Deserialize, PartialEq, Eq)]
// enum PropResults {
//     Images(),
//     ImageInfo(),
//     Categories(),
//     PageImages(),
// }

// #[derive(Debug, Deserialize, PartialEq, Eq)]
// struct PageGeneral {
//     pageid: u32,
//     ns: u32,
//     title: String,
// }

// #[derive(Debug, Deserialize, PartialEq, Eq)]
// #[serde(rename_all = "lowercase")]
// enum SubContinue {
//     ImContinue(String),
// }

// #[derive(Debug, Deserialize, PartialEq, Eq)]
// struct ImageItem {
//     ns: u32,
//     title: String,
// }

// #[derive(Debug, Deserialize)]
// struct ImageInfo {
//     #[serde(rename = "canonicaltitle")]
//     title: String,
//     size: usize,
//     width: usize,
//     height: usize,
//     url: String,
// }
