use crate::NAMESPACE;
use crate::response::{de_helpers::*, items::*};
use serde::de::Deserialize;
use std::collections::HashMap;

#[derive(Debug)]
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
            pageids: helper
                .query
                .pageids
                .map(|ids| ids.into_iter().filter_map(TryInto::<u32>::try_into))
                .collect(), // only collect valid pageids (id >= 0)
            results: helper
                .query
                .pages
                .into_values()
                .filter_map(QueryResult::from_helper)
                .collect(), // only collect valid results, see QueryResult::from_helper for details
        })
    }
}

#[derive(Debug)]
pub struct QueryResult {
    pub pageid: u32,
    pub ns: NAMESPACE,
    pub title: String,
    pub categories: Option<CategoriesProp>,
    pub categoryinfo: Option<CategoryInfoProp>,
    pub images: Option<ImagesProp>,
    pub imageinfo: Option<ImageInfoProp>,
    pub pageimages: Option<PageImagesProp>,
    pub info: Option<InfoProp>,
}

impl QueryResult {
    pub(crate) fn from_helper(helper: QueryResultHelper) -> Option<Self> {
        match helper {
            QueryResultHelper {
                pageid: Some(pageid_),
                ns: Some(ns_),
                title: Some(title_),
                categories: Some(categories_),
                categoryinfo: Some(categoryinfo_),
                images: Some(images_),
                imageinfo: Some(imageinfo_),
                pageimages: Some(pageimages_),
                info: Some(info_),
            } => Some(QueryResult {
                pageid: pageid_.try_into()?, // result is invalid if pageid < 0
                ns: ns_.try_into()?, // result is invalid if ns (namespace) does not map to a valid namespace (see crate::NAMESPACE)
                title: title_,

                // empty property fields (determined by PropResult::empty) are deserialized as None
                //
                categories: if categories_.empty() {
                    None
                } else {
                    Some(categories_)
                },
                categoryinfo: if categoryinfo_.empty() {
                    None
                } else {
                    Some(categoryinfo_)
                },
                images: if images_.empty() { None } else { Some(images_) },
                imageinfo: if imageinfo_.empty() {
                    None
                } else {
                    Some(imageinfo_)
                },
                pageimages: if pageimages_.empty() {
                    None
                } else {
                    Some(pageimages_)
                },
                info: if info_.empty() { None } else { Some(info_) },
                //
                // end property fields check
            }),
            _ => None,
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
