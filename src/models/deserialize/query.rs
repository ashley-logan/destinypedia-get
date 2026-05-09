use crate::models::deserialize::{de_helpers::*, prop_results::*};
use serde::Deserialize;
use serde::de;
use std::collections::HashMap;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Continue {
    #[serde(rename = "continue")]
    pub contin: String,
    #[serde(flatten)]
    pub sub_cont: HashMap<String, String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Query<T: PropResults> {
    // will usually deserialize from 'pageid': [items] or 'pageid': {item_fields}
    pub pages: Option<HashMap<String, QueryResult<T>>>,
}

impl<'de, T: PropResults + Deserialize<'de>> Deserialize<'de> for Query<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper: QueryHelper<T> = QueryHelper::deserialize(deserializer)?;

        let no_de: bool;
        if let Some(pgs) = &helper.pages {
            no_de = pgs.contains_key("-1") || pgs.iter().all(|(_, v)| v.items.is_none());
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
pub struct QueryResult<T: PropResults> {
    pub pageid: Option<usize>,
    pub ns: Option<usize>,
    pub title: Option<String>,
    pub missing: Option<String>,
    pub items: Option<T>,
}

impl<'de, T: PropResults + Deserialize<'de>> Deserialize<'de> for QueryResult<T> {
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

#[derive(Debug, PartialEq, Eq)]
pub struct IndiscriminateQueryResult {
    pub pageid: u32,
    pub ns: crate::models::NAMESPACE,
    pub title: String,
    pub categories: Option<CategoriesProp>,
    pub categoryinfo: Option<CategoryInfoProp>,
    pub images: Option<ImagesProp>,
    pub imageinfo: Option<ImageInfoProp>,
    pub pageimages: Option<PageImagesProp>,
    pub info: Option<InfoProp>,
}

impl<'de> Deserialize<'de> for IndiscriminateQueryResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper: IndiscriminateQueryResultHelper =
            IndiscriminateQueryResultHelper::deserialize(deserializer)?;

        let id = helper
            .pageid
            .ok_or_else(|| de::Error::custom("invalid request"))?;

        let pageid: u32 = id
            .try_into()
            .map_err(|_| de::Error::custom("invalid request, no results returned"))?;

        let ns = helper.ns.ok_or_else(|| de::Error::custom("invalid response, no namespace found"))?;

        let title = helper.title.ok_or_else(|| de::Error::custom("invalid response, no title found"))?;


        let mut r = IndiscriminateQueryResult {
            pageid,
            ns,
            title,
            categories: None,
            categoryinfo: None,
            images: None,
            pageimages: None,
            imageinfo: None,
            info: None,
        };

        if let Some(x) = &helper.categories {
            if !x.inner_all_none() {
                r.categories = helper.categories;
            }
        }

        if let Some(x) = &helper.categoryinfo {
            if !x.inner_all_none() {
                r.categoryinfo = helper.categoryinfo;
            }
        }

        if let Some(x) = &helper.images {
            if !x.inner_all_none() {
                r.images = helper.images;
            }
        }

        if let Some(x) = &helper.imageinfo {
            if !x.inner_all_none() {
                r.imageinfo = helper.imageinfo;
            }
        }

        if let Some(x) = &helper.pageimages {
            if !x.inner_all_none() {
                r.pageimages = helper.pageimages;
            }
        }

        if let Some(x) = &helper.info {
            if !x.inner_all_none() {
                r.info = helper.info;
            }
        }

        Ok(r)
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
    fn test_query_result1() {
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
    fn test_query_result2() {
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
    fn test_query1() {
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

        let exp: Query<prop_results::PageInfo> = Query { pages: None };

        assert_eq!(
            from_value::<Query<prop_results::PageInfo>>(control)
                .expect("Failed to convert control to Query<PageInfo>"),
            exp
        )
    }

    #[test]
    fn test_query2() {
        let control = json!({
            "pages": {
                "696": {
                    "pageid": 696,
                    "ns": 14,
                    "title": "Category:Enemies",
                    "categoryinfo": {
                        "size": 2966,
                        "pages": 2961,
                        "files": 3,
                        "subcats": 2
                    }
                }
            }
        });

        let mut map: HashMap<String, QueryResult<prop_results::CategoryInfo>> = HashMap::new();
        map.insert(
            "696".into(),
            QueryResult {
                pageid: Some(696),
                ns: Some(14),
                title: Some("Category:Enemies".into()),
                missing: None,
                items: Some(prop_results::CategoryInfo {
                    categoryinfo: items::CatgeoryInfoItem {
                        size: Some(2966),
                        pages: Some(2961),
                        files: Some(3),
                        subcats: Some(2),
                    },
                }),
            },
        );

        let exp: Query<prop_results::CategoryInfo> = Query { pages: Some(map) };

        assert_eq!(
            from_value::<Query<prop_results::CategoryInfo>>(control)
                .expect("Failed to convert control to Query<CategoryInfo>"),
            exp
        );
    }

    #[test]
    fn test_indiscriminate_query_result1() {
        let control = json!({
            "pageid": 50116,
            "ns": 14,
            "title": "Category:ARG",
            "categoryinfo": {
                "size": 3,
                "pages": 3,
                "files": 0,
                "subcats": 0
            },
            "categories": [
                {
                    "ns": 14,
                    "title": "Category:Lore"
                },
                {
                    "ns": 14,
                    "title": "Category:Promotional material"
                }
            ]
        });

        let cats = prop_results::Categories {
            categories: vec![
                items::CategoryItem {
                    ns: 14,
                    title: "Category:Lore".into(),
                },
                items::CategoryItem {
                    ns: 14,
                    title: "Category:Promotional material".into(),
                },
            ],
        };

        let cat_info = prop_results::CategoryInfo {
            categoryinfo: items::CatgeoryInfoItem {
                size: Some(3),
                pages: Some(3),
                files: Some(0),
                subcats: Some(0),
            },
        };

        let exp = IndiscriminateQueryResult {
            pageid: Some(50116),
            ns: Some(14),
            title: Some("Category:ARG".into()),
            missing: None,
            categories: Some(cats),
            categoryinfo: Some(cat_info),
            images: None,
            pageimages: None,
            imageinfo: None,
            info: None,
        };

        assert_eq!(
            from_value::<IndiscriminateQueryResult>(control)
                .expect("Failed to convert control to IndiscriminateQueryResult"),
            exp
        );
    }
}
