use super::prop_results::PropResults;
use super::query::Continue;
use crate::models::serialize::NAMESPACE;
use serde::Deserialize;
use serde_with::{DefaultOnError, DeserializeAs, TryFromInto, serde_as};
// use super::{Categories, CategoryInfo, ImageInfo, Images, PageImages, PageInfo};
use super::items;
use std::collections::HashMap;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct InfallibleResponse {
    pub cont: Option<Continue>,
    pub query: InfallibleQuery,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct InfallibleQuery {
    pub pages: HashMap<String, InfallibleQueryResult>,
}



#[serde_as]
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct InfallibleQueryResult {
    pub pageid: Option<u64>,
    #[serde_as(as = "DefaultOnError<Option<TryFromInto<u16>>>")]
    pub ns: Option<NAMESPACE>,
    pub title: Option<String>,
    pub missing: Option<String>,
    pub categories: Option<CategoriesWrapper>,
    #[serde(flatten)]
    pub categoryinfo: Option<CategoryInfoWrapper>,
    pub images: Option<ImagesWrapper>,
    #[serde(flatten)]
    pub pageimages: Option<PageImagesWrapper>,
    pub imageinfo: Option<ImageInfoWrapper>,
    #[serde(flatten)]
    pub info: Option<InfoWrapper>,
}

#[derive(Debug)]
pub struct QueryResult {
    pub pageid: u64,
    pub ns: NAMESPACE,
    pub title: String,
    pub categories: Vec<items::CategoryItem>,
    pub categoryinfo: Option<items::CatgeoryInfoItem>,
    pub images: Vec<items::ImageItem>,
    pub pageimages: Option<items::PageImageItem>,
    pub imageinfo: Option<items::ImageInfoItem>,
    pub info: Option<items::PageInfoItem>,
}

impl QueryResult {
    pub fn into_iter_categories(self) -> impl IntoIterator<Item = items::CategoryItem> {
        self.categories.into_iter()
    }

    pub fn into_iter_images(self) -> impl IntoIterator<Item = items::ImageItem> {
        self.images.into_iter()
    }
}

impl TryFrom<InfallibleQueryResult> for QueryResult {
    type Error = crate::Error;
    fn try_from(value: InfallibleQueryResult) -> Result<Self, Self::Error> {
        match (value.pageid, value.ns, value.title, value.missing) {
            (_, _, _, Some(_)) => Err(crate::Error::TryIntoQueryResult),
            (Some(pid), Some(n), Some(t), None) => Ok(QueryResult {
                pageid: pid,
                ns: n,
                title: t,
                categories: value.categories.map_or_else(Vec::new, |c| c.categories),
                categoryinfo: value.categoryinfo,
                images: value.images.map_or_else(Vec::new, |i| i.images),
                pageimages: value.pageimages.map(|pi| pi.pageimages),
                imageinfo: value.imageinfo.map(|ii| ii.imageinfo[0].clone()),
                info: value.info,
            }),
            _ => Err(crate::Error::TryIntoQueryResult),
        }
    }
}

// pub(crate) struct IndiscriminateQueryHelper {
//     pub(crate) pages: HashMap<String, IndiscriminateQueryResult>,
// }

// pub(crate) struct IndiscriminateResponseHelper {
//     #[serde(rename = "continue")]
//     pub(crate) cont: Option<Continue>,
//     pub(crate) query: IndiscriminateQueryHelper,
// }

pub struct QueryResponseFromHelper;

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct QueryResponse {
    pub continue_: Option<Continue>,
    #[serde_as(as = "QueryResponseFromHelper")]
    #[serde(flatten)]
    pub results: Vec<QueryResult>,
}

impl QueryResponse {
    pub fn get_continue(&self) -> Option<(&String, &String)> {
        if let Some(c) = &self.continue_ {
            c.sub_cont.iter().by_ref().next()
        } else {
            None
        }
    }

    pub fn into_iter_results(self) -> impl IntoIterator<Item = QueryResult> {
        self.results.into_iter()
    }
}

impl<'de> DeserializeAs<'de, Vec<QueryResult>> for QueryResponseFromHelper {
    fn deserialize_as<D>(deserializer: D) -> Result<Vec<QueryResult>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper: InfallibleResponse = InfallibleResponse::deserialize(deserializer)?;

        let mut resp: Vec<QueryResult> = vec![];

        if let Some((k, v)) = helper.query.pages.iter().next() {
            if k == "-1" {
                dbg!(format!("de failed => page id == -1"));
                return Ok(resp);
            }
            if v.missing.is_some() {
                dbg!(format!("de failed => missing is some"));
                return Ok(resp);
            }

            if v.pageid
                .is_none_or(|i| k.parse::<u64>().ok().is_none_or(|ki| i != ki))
            {
                dbg!(format!("de failed => pageids none or not equal"));
                return Ok(resp);
            }
        } else {
            dbg!(format!("de failed => no results in pages"));
            return Ok(resp);
        }

        resp.extend(
            helper
                .query
                .pages
                .into_values()
                .filter_map(|v| v.try_into().ok()),
        );

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::Read;
    use std::path::{Path, PathBuf};

    static DATA_DIR: &str = "data/example_responses";

    fn get_data_dir() -> PathBuf {
        let mut dir: PathBuf = env::current_dir().unwrap();
        dir.push(Path::new(DATA_DIR));
        dir
    }

    #[test]
    fn test_tryinto_query_result() {
        let mut fpath = get_data_dir();
        fpath.push(Path::new("ok/prop_pageimages_original|name.json"));

        let mut f = File::open(fpath).unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();

        let resp: QueryResponse = serde_json::from_str(buf.as_str()).unwrap();

        dbg!(&resp);
    }

    #[test]
    fn test_queryresponse_allimages() {
        let mut fpath = get_data_dir();
        fpath.push(Path::new(
            "ok/generator_allcategories_prop_categories|categoryinfo.json",
        ));

        let mut f = File::open(fpath).unwrap();
        let mut buf = String::new();

        f.read_to_string(&mut buf).unwrap();

        let resp: QueryResponse = serde_json::from_str(buf.as_str()).unwrap();

        dbg!(&resp);
    }
}
