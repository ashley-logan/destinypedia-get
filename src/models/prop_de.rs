use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct PropResponse {
    #[serde(rename = "continue")]
    cont: Option<Continue>,
    query: Query<T>,
}
#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Continue {
    #[serde(rename = "continue")]
    contin: String,
    #[serde(flatten)]
    sub_cont: SubContinue,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Query<T> {
    pages: Option<HashMap<String, Page<T>>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
enum PropResults {
    Images(Vec<ImageItem>),
    ImageInfo(Vec<ImageInfoItem>),
    Categories(Vec<CategoryItem>),
    #[serde(untagged)]
    PageImages {
        original: PageImageItem,
        pageimage: String,
    },
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Page<T> {
    pageid: u32,
    ns: u32,
    title: String,
    #[serde(flatten)]
    items: Option<HashMap<String, T>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum SubContinue {
    ImContinue(String),
    IiContinue(String),
    PiContinue(String),
    ClContinue(String),
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Images {
    images: 
}
#[derive(Debug, Deserialize, PartialEq, Eq)]
struct ImageItem {
    ns: u32,
    title: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ImageInfo {
    imageinfo: Vec<ImageInfoItem>
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct ImageInfoItem {
    #[serde(rename = "canonicaltitle")]
    title: String,
    size: usize,
    width: usize,
    height: usize,
    url: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Categories {
    categories: Vec<CategoryItem>
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct CategoryItem {
    ns: u32,
    title: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct PageImages {
    pageimages: Vec<PageImageItem>
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct PageImageItem {
    source: String,
    width: usize,
    height: usize,
}
