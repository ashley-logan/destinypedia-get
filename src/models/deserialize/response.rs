use crate::models::deserialize::de_helpers::*;
use crate::models::deserialize::prop_results::*;
use crate::models::deserialize::query::*;
use serde::de::Deserialize;
use std::collections::HashMap;

pub trait ResponseTrait {
    fn get_continue_param(&self) -> Option<(&str, &str)>;
}

#[derive(Debug, derive_more::PartialEq, derive_more::Eq)]
pub struct Response<T: PropResults> {
    // in helper: #[serde(rename = 'continue')]
    cont: Option<Continue>,
    query: Option<Query<T>>,
}

#[derive(Debug, derive_more::PartialEq, derive_more::Eq)]
pub struct IndiscriminateResponse {
    pub cont: Option<Continue>,
    pub results: HashMap<String, IndiscriminateQueryResult>,
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
            results: helper.query.pages,
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
    pub fn get_results(&self) -> &HashMap<String, IndiscriminateQueryResult> {
        &self.results
    }
}

impl<'de, T: PropResults + Deserialize<'de>> Deserialize<'de> for Response<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper: ResponseHelper<T> = ResponseHelper::deserialize(deserializer)?;

        let no_de: bool;
        if let Some(q) = &helper.query {
            no_de = q.pages.is_none();
        } else {
            no_de = true;
        }

        if no_de {
            Ok(Response {
                cont: None,
                query: None,
            })
        } else {
            Ok(Response {
                cont: helper.cont,
                query: helper.query,
            })
        }
    }
}

impl<T: PropResults> ResponseTrait for Response<T> {
    fn get_continue_param(&self) -> Option<(&str, &str)> {
        if let Some(c) = &self.cont {
            for (k, v) in &c.sub_cont {
                if k.ends_with("continue") {
                    return Some((k, v));
                }
            }
            None
        } else {
            None
        }
    }
}

impl<T: PropResults> Response<T> {
    pub fn get_results(&self) -> Option<&HashMap<String, QueryResult<T>>> {
        if let Some(q) = &self.query {
            if let Some(map) = &q.pages {
                return Some(map);
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
    use std::io::BufReader;
    use std::path::{Path, PathBuf};
    use std::str::FromStr;

    static PATH_STR: &str = "./data/example_responses/ok";
    fn get_data_dir() -> PathBuf {
        let mut p = env::current_dir().unwrap();
        p.push(Path::new(PATH_STR));
        p
    }

    #[test]
    fn test_indiscriminate_resp1() {
        let mut p = get_data_dir();
        p.push(Path::new("generator_allimages.json"));
        let f = File::open(p).unwrap();

        let mut rdr = BufReader::new(f);

        let resp: Response<ImageInfo> = from_reader(rdr).unwrap();

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
}
