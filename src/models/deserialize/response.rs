use crate::models::deserialize::de_helpers::*;
use crate::models::deserialize::prop_results::*;
use crate::models::deserialize::query::*;
use serde::de::Deserialize;
use std::collections::HashMap;

#[derive(Debug, derive_more::PartialEq, derive_more::Eq)]
pub struct QueryResponse {
    pub pageids: Option<Vec<u32>>,
    pub results: Vec<QueryResult>,
}

impl<'de> Deserialize<'de> for QueryResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper: QueryResponseHelper = QueryResponseHelper::deserialize(deserializer)?;

        Ok(QueryResponse {
            pageids: helper.query.pageids,
            results: helper.query.pages.into_values().collect(),
        })
    }
}

impl QueryResponse {
    pub fn get_results(&self) -> &[QueryResult] {
        self.results.as_slice()
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
    use serde_test::assert_de_tokens;
    use std::env;
    use std::fs::File;
    use std::io::{BufReader, Read};
    use std::path::{Path, PathBuf};
    use std::str::FromStr;

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

        let resp: QueryResponse = from_reader(rdr).unwrap();
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
    fn test_resp3() {
        let mut p = get_data_dir();
        p.push(Path::new("prop_pageimages_original|name.json"));

        let f = File::open(p).unwrap();

        let mut rdr = BufReader::new(f);

        let resp: QueryResponse = from_reader(rdr).unwrap();

        dbg!(&resp);
    }

    #[test]
    fn test_fail_resp1() {
        let mut p = get_data_fail_dir();
        p.push(Path::new("prop_info.json"));

        let f = File::open(p).unwrap();

        let mut rdr = BufReader::new(f);

        assert!(from_reader::<_, QueryResponse>(rdr).is_err());
    }
}
