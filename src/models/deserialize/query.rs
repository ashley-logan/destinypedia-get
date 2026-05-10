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

// #[derive(Debug, PartialEq, Eq)]
// pub struct Query<T: PropResults> {
//     // will usually deserialize from 'pageid': [items] or 'pageid': {item_fields}
//     pub pages: Option<HashMap<String, QueryResult<T>>>,
// }

// impl<'de, T: PropResults + Deserialize<'de>> Deserialize<'de> for Query<T> {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let helper: QueryHelper<T> = QueryHelper::deserialize(deserializer)?;

//         let no_de: bool;
//         if let Some(pgs) = &helper.pages {
//             no_de = pgs.contains_key("-1") || pgs.iter().all(|(_, v)| v.items.is_none());
//         } else {
//             no_de = true;
//         }

//         if no_de {
//             Ok(Query { pages: None })
//         } else {
//             Ok(Query {
//                 pages: helper.pages,
//             })
//         }
//     }
// }

// #[derive(Debug, PartialEq, Eq)]
// pub struct QueryResult<T: PropResults> {
//     pub pageid: Option<usize>,
//     pub ns: Option<usize>,
//     pub title: Option<String>,
//     pub missing: Option<String>,
//     pub items: Option<T>,
// }

// impl<'de, T: PropResults + Deserialize<'de>> Deserialize<'de> for QueryResult<T> {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let helper: QueryResultHelper<T> = QueryResultHelper::deserialize(deserializer)?;

//         let no_de: bool = {
//             match (&helper.items, &helper.missing) {
//                 (_, Some(_)) => true, // is 'missing' field is some, don't deserialize items
//                 (Some(x), _) => x.all_empty(), // if all items are empty, don't deserialize items
//                 (None, _) => true,    // is items itself is none, don't deserialize items
//             }
//         };

//         if no_de {
//             Ok(QueryResult {
//                 pageid: helper.pageid,
//                 ns: helper.ns,
//                 title: helper.title,
//                 missing: helper.missing,
//                 items: None,
//             })
//         } else {
//             Ok(QueryResult {
//                 pageid: helper.pageid,
//                 ns: helper.ns,
//                 title: helper.title,
//                 missing: helper.missing,
//                 items: helper.items,
//             })
//         }
//     }
// }

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

}
