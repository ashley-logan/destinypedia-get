use crate::models::deserialize::{de_helpers::*, prop_results::*};
use serde::Deserialize;
use serde::de;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub struct QueryResult {
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

impl QueryResult {
    pub(crate) fn from_helper(helper: QueryResultHelper) -> Option<Self> {
        let mut r: QueryResult = match helper {
            QueryResultHelper {
                pageid: Some(pageid_),
                ns: Some(ns_),
                title: Some(title_),
                ..
            } if TryInto::<u32>::try_into(pageid_).is_ok() => QueryResult {
                pageid: pageid_.try_into().unwrap(),
                ns: ns_,
                title: title_,
                categories: None,
                categoryinfo: None,
                images: None,
                imageinfo: None,
                pageimages: None,
                info: None,
            },
            _ => return None,
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

        Some(r)
    }
}

// impl<'de> Deserialize<'de> for QueryResult {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let helper: QueryResultHelper = QueryResultHelper::deserialize(deserializer)?;

//         let id = helper
//             .pageid
//             .ok_or_else(|| de::Error::custom("invalid request"))?;

//         let pageid: u32 = id
//             .try_into()
//             .map_err(|_| de::Error::custom("invalid request, no results returned"))?;

//         let ns = helper
//             .ns
//             .ok_or_else(|| de::Error::custom("invalid response, no namespace found"))?;

//         let title = helper
//             .title
//             .ok_or_else(|| de::Error::custom("invalid response, no title found"))?;

//         let mut r = QueryResult {
//             pageid,
//             ns,
//             title,
//             categories: None,
//             categoryinfo: None,
//             images: None,
//             pageimages: None,
//             imageinfo: None,
//             info: None,
//         };

//         if let Some(x) = &helper.categories {
//             if !x.inner_all_none() {
//                 r.categories = helper.categories;
//             }
//         }

//         if let Some(x) = &helper.categoryinfo {
//             if !x.inner_all_none() {
//                 r.categoryinfo = helper.categoryinfo;
//             }
//         }

//         if let Some(x) = &helper.images {
//             if !x.inner_all_none() {
//                 r.images = helper.images;
//             }
//         }

//         if let Some(x) = &helper.imageinfo {
//             if !x.inner_all_none() {
//                 r.imageinfo = helper.imageinfo;
//             }
//         }

//         if let Some(x) = &helper.pageimages {
//             if !x.inner_all_none() {
//                 r.pageimages = helper.pageimages;
//             }
//         }

//         if let Some(x) = &helper.info {
//             if !x.inner_all_none() {
//                 r.info = helper.info;
//             }
//         }

//         Ok(r)
//     }
// }

#[cfg(test)]
mod tests {}
