use crate::models::deserialize::de_helpers::*;
use crate::models::deserialize::prop_results::*;
use crate::models::deserialize::query::*;
use serde::de::Deserialize;
use std::collections::HashMap;

pub trait ResponseTrait {
    fn get_continue_param(&self) -> Option<(&str, &str)>;
}



#[derive(Debug, derive_more::PartialEq, derive_more::Eq)]
pub struct IndiscriminateResponse {
    pub cont: Option<Continue>,
    pub results: Vec<IndiscriminateQueryResult>,
}

impl<'de> Deserialize<'de> for IndiscriminateResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper: IndiscriminateResponseHelper =
            IndiscriminateResponseHelper::deserialize(deserializer)?;


        Ok(IndiscriminateResponse {
            cont: helper.cont,
            results: helper.query.pages.into_values().collect(),
        })
    }
}

impl ResponseTrait for IndiscriminateResponse {
    fn get_continue_param(&self) -> Option<(&str, &str)> {
        if let Some(c) = &self.cont {
            for (k, v) in &c.sub_cont {
                if k.ends_with("continue") {
                    return Some((k, v));
                }
            }
        }
        None
    }
}

impl IndiscriminateResponse {
    pub fn get_results(&self) -> &[IndiscriminateQueryResult] {
        self.results.as_slice()
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

        let resp: IndiscriminateResponse = from_reader(rdr).unwrap();
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

        let resp: IndiscriminateResponse = from_reader(rdr).unwrap();

        dbg!(&resp);
    }

    #[test]
    fn test_resp3() {
        let mut p = get_data_dir();
        p.push(Path::new("prop_pageimages_original|name.json"));

        let f = File::open(p).unwrap();

        let mut rdr = BufReader::new(f);

        let resp: IndiscriminateResponse = from_reader(rdr).unwrap();

        dbg!(&resp);
    }

    #[test]
    fn test_fail_resp1() {
         let mut p = get_data_fail_dir();
        p.push(Path::new("prop_info.json"));

        let f = File::open(p).unwrap();

        let mut rdr = BufReader::new(f);

        assert!(from_reader::<_, IndiscriminateResponse>(rdr).is_err());
    }
}
