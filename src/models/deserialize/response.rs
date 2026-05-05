use crate::models::deserialize::de_helpers::*;
use crate::models::deserialize::prop_results::*;
use crate::models::deserialize::query::*;
use serde::de::Deserialize;
use std::collections::HashMap;

pub trait ResponseTrait {
    fn get_continue_param(&self) -> Option<(&str, &str)>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct Response<T: PropResults> {
    // in helper: #[serde(rename = 'continue')]
    cont: Option<Continue>,
    query: Option<Query<T>>,
}

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
