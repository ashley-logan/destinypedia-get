use crate::NAMESPACE;
use crate::response::{de_helpers::*, items::*};
use serde::de::Deserialize;
use std::collections::HashMap;

#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(from = "QueryResponseHelper")]
pub struct QueryResponse {
    pub continue_: Option<Continue>,
    pub pageids: Option<Vec<i32>>,
    pub results: Vec<QueryResult>,
}

impl QueryResponse {
    pub fn get_continue_tuple(&self) -> Option<(&String, &String)> {
        match &self.continue_ {
            Some(Continue { sub_cont, .. }) => sub_cont.iter().next(),
            _ => None,
        }
    }
}

impl From<QueryResponseHelper> for QueryResponse {
    fn from(helper: QueryResponseHelper) -> Self {
        QueryResponse {
            continue_: helper.continue_,
            pageids: helper.query.pageids,
            results: helper
                .query
                .pages
                .into_values()
                .filter_map(|r| r.try_into().ok())
                .collect(), // only collect valid results, see QueryResult::from_helper for details
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct QueryResult {
    pub pageid: i32,
    pub ns: NAMESPACE,
    pub title: String,
    pub categories: Option<Categories>,
    pub categoryinfo: Option<CategoryInfo>,
    pub images: Option<Images>,
    pub imageinfo: Option<ImageInfo>,
    pub pageimages: Option<PageImages>,
    pub info: Option<Info>,
}

impl TryFrom<QueryResultHelper> for QueryResult {
    type Error = super::error::ResponseError;
    fn try_from(helper: QueryResultHelper) -> Result<Self, Self::Error> {
        match helper {
            QueryResultHelper {
                pageid: Some(pageid),
                ns: Some(ns),
                title: Some(title),
                categories,
                categoryinfo,
                images,
                imageinfo,
                pageimages,
                info,
            } => Ok(QueryResult {
                pageid, // result is invalid if pageid < 0
                ns,
                title,

                // empty property fields (determined by PropResult::empty) are deserialized as None
                //
                categories: match &categories {
                    Some(x) if !x.empty() => categories,
                    _ => None,
                },
                categoryinfo: match &categoryinfo {
                    Some(x) if !x.empty() => categoryinfo,
                    _ => None,
                },
                images: match &images {
                    Some(x) if !x.empty() => images,
                    _ => None,
                },
                imageinfo: match &imageinfo {
                    Some(x) if !x.empty() => imageinfo,
                    _ => None,
                },
                pageimages: match &pageimages {
                    Some(x) if !x.empty() => pageimages,
                    _ => None,
                },
                info: match &info {
                    Some(x) if !x.empty() => info,
                    _ => None,
                },
                //
                // end property fields check
            }),
            _ => Err(super::error::ResponseError::MissingField),
        }
    }
}

#[derive(Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "continue")]
pub struct Continue {
    #[serde(rename = "continue")]
    pub contin: String,
    #[serde(flatten)]
    pub sub_cont: HashMap<String, String>,
}

impl Continue {
    pub fn get_continue_pair(&self) -> Option<(&str, &str)> {
        for (k, v) in &self.sub_cont {
            if k.ends_with("continue") {
                return Some((k.as_str(), v.as_str()));
            }
        }
        None
    }

    pub fn into_tuple(self) -> Option<(String, String)> {
        for (k, v) in self.sub_cont {
            if k.ends_with("continue") {
                return Some((k, v));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_reader;
    use serde_test::{Token, assert_de_tokens};
    use std::env;
    use std::fs::File;
    use std::io::{BufReader, Read};
    use std::path::{Path, PathBuf};

    static PATH_STR: &str = "./data/example_responses/ok";
    fn get_data_dir() -> PathBuf {
        let mut p = env::current_dir().unwrap();
        p.push(Path::new(PATH_STR));
        p
    }

    fn get_data_fail_dir() -> PathBuf {
        let mut p = env::current_dir().unwrap();
        p.push(Path::new("./data/example_responses/fail"));
        p
    }

    #[test]
    fn test_resp1() {
        let mut p = get_data_dir();
        p.push(Path::new("generator_allimages_prop_imageinfo.json"));

        let mut rdr = BufReader::new(File::open(p).unwrap());

        let mut resp: QueryResponse = from_reader(rdr).unwrap();
        dbg!(&resp);
    }

    #[test]
    fn test_resp2() {
        let mut p = get_data_dir();
        p.push(Path::new(
            "generator_allcategories_prop_categories|categoryinfo.json",
        ));
        let f = File::open(p).unwrap();

        let mut rdr = BufReader::new(f);

        let resp: QueryResponse = from_reader(rdr).unwrap();

        dbg!(&resp);
    }

    #[test]
    fn test_invalid_results_resp1() {
        let mut p = get_data_fail_dir();
        p.push(Path::new("prop_info.json"));

        let f = File::open(p).unwrap();

        let mut rdr = BufReader::new(f);

        assert!(from_reader::<_, QueryResponse>(rdr).is_ok());
    }
}
