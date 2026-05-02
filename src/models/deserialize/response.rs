use crate::models::deserialize::de_helpers::*;
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
pub(crate) struct Continue {
    #[serde(rename = "continue")]
    pub(crate) contin: String,
    #[serde(flatten)]
    pub(crate) sub_cont: HashMap<String, String>,
}

#[derive(Debug, PartialEq, Eq)]
struct Query<T: prop_results::PropResults> {
    // will usually deserialize from 'pageid': [items] or 'pageid': {item_fields}
    pages: Option<HashMap<String, QueryResult<T>>>,
}

impl<'de, T: prop_results::PropResults + Deserialize<'de>> Deserialize<'de> for Query<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper: QueryHelper<T> = QueryHelper::deserialize(deserializer)?;

        let no_de: bool;
        if let Some(pgs) = &helper.pages {
            no_de = pgs.contains_key("-1") || pgs.iter().all(|(k, v)| v.items.is_none());
        } else {
            no_de = true;
        }

        if no_de {
            Ok(Query { pages: None })
        } else {
            Ok(Query {
                pages: helper.pages,
            })
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct QueryResult<T: prop_results::PropResults> {
    pub(crate) pageid: Option<usize>,
    pub(crate) ns: Option<usize>,
    pub(crate) title: Option<String>,
    pub(crate) missing: Option<String>,
    pub(crate) items: Option<T>,
}

impl<'de, T: prop_results::PropResults + Deserialize<'de>> Deserialize<'de> for QueryResult<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper: QueryResultHelper<T> = QueryResultHelper::deserialize(deserializer)?;

        let no_de: bool = {
            match (&helper.items, &helper.missing) {
                (_, Some(_)) => true, // is 'missing' field is some, don't deserialize items
                (Some(x), _) => x.all_empty(), // if all items are empty, don't deserialize items
                (None, _) => true,    // is items itself is none, don't deserialize items
            }
        };

        if no_de {
            Ok(QueryResult {
                pageid: helper.pageid,
                ns: helper.ns,
                title: helper.title,
                missing: helper.missing,
                items: None,
            })
        } else {
            Ok(QueryResult {
                pageid: helper.pageid,
                ns: helper.ns,
                title: helper.title,
                missing: helper.missing,
                items: helper.items,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::deserialize::{items, prop_results};
    use serde_json::{from_value, json};
    use std::collections::HashMap;

    #[test]
    fn test_continue() {
        let control = json!({
            "gapcontinue": "\"Laughing_Behind_Your_Back\"",
            "continue": "gapcontinue||"
        });

        let mut map: HashMap<String, String> = HashMap::new();
        map.insert(
            "gapcontinue".to_string(),
            "\"Laughing_Behind_Your_Back\"".to_string(),
        );
        let exp = Continue {
            contin: "gapcontinue||".into(),
            sub_cont: map,
        };

        dbg!(&exp);
        dbg!(&control);
        assert_eq!(
            from_value::<Continue>(control).expect("Failed to convert control to Continue"),
            exp
        );
    }

    #[test]
    fn test_result_base1() {
        let control = json!({
            "pageid": 1034,
            "ns": 0,
            "title": "Hive",
            "contentmodel": "wikitext",
            "pagelanguage": "en",
            "pagelanguagehtmlcode": "en",
            "pagelanguagedir": "ltr",
            "touched": "2026-04-12T04:03:19Z",
            "lastrevid": 390101,
            "length": 220921
        });

        let exp: QueryResult<prop_results::PageInfo> = QueryResult {
            pageid: Some(1034),
            ns: Some(0),
            title: Some("Hive".into()),
            missing: None,
            items: Some(prop_results::PageInfo {
                pageinfo: items::PageInfoItem {
                    contentmodel: Some("wikitext".into()),
                    length: Some(220921),
                },
            }),
        };

        assert_eq!(
            from_value::<QueryResult<prop_results::PageInfo>>(control)
                .expect("Failed to convert control to QueryResult<PageInfo>"),
            exp
        )
    }

    #[test]
    fn test_result_base2() {
        let control = json!({
            "pageid": 1034,
            "ns": 0,
            "title": "Hive",
            "images": [
                {
                    "ns": 6,
                    "title": "File:Alakhul.jpg"
                },
                {
                    "ns": 6,
                    "title": "File:ArcS.png"
                },
            ]
        });

        let exp: QueryResult<prop_results::Images> = QueryResult {
            pageid: Some(1034),
            ns: Some(0),
            title: Some("Hive".into()),
            missing: None,
            items: Some(prop_results::Images {
                images: vec![
                    items::ImageItem {
                        ns: 6,
                        title: "File:Alakhul.jpg".into(),
                    },
                    items::ImageItem {
                        ns: 6,
                        title: "File:ArcS.png".into(),
                    },
                ],
            }),
        };

        assert_eq!(
            from_value::<QueryResult<prop_results::Images>>(control)
                .expect("Failed to convert control to QueryResult<Images>"),
            exp
        )
    }

    #[test]
    fn test_query_hashmap() {
        let control = json!({
            "pages":
                {
                    "15398": {
                        "ns": 0,
                        "title": "Heve",
                        "missing": "",
                        "contentmodel": "wikitext",
                        "pagelanguage": "en",
                        "pagelanguagehtmlcode": "en",
                        "pagelanguagedir": "ltr"
                    },
                    "-1": {
                        "ns": 0,
                        "title": "Heve",
                        "missing": ""
                    }
                }
        });

        let mut map: HashMap<String, QueryResult<prop_results::PageInfo>> = HashMap::new();

        map.insert(
            "-1".into(),
            QueryResult {
                pageid: None,
                ns: Some(0),
                title: Some("Heve".into()),
                missing: Some("".into()),
                items: None,
            },
        );

        map.insert(
            "15398".into(),
            QueryResult {
                pageid: None,
                ns: Some(0),
                title: Some("Heve".into()),
                missing: Some("".into()),
                items: Some(prop_results::PageInfo {
                    pageinfo: items::PageInfoItem {
                        contentmodel: Some("wikitext".into()),
                        length: None,
                    },
                }),
            },
        );

        let exp: Query<prop_results::PageInfo> = Query { pages: Some(map) };

        assert_eq!(
            from_value::<Query<prop_results::PageInfo>>(control)
                .expect("Failed to convert control to Query<PageInfo>"),
            exp
        )
    }
}
