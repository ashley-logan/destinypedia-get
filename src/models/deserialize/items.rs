use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ImageItem {
    pub ns: u32,
    pub title: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ImageInfoItem {
    pub canonicaltitle: Option<String>,
    pub size: Option<usize>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub url: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct CategoryItem {
    pub ns: u32,
    pub title: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct CatgeoryInfoItem {
    pub size: Option<usize>,
    pub pages: Option<usize>,
    pub files: Option<usize>,
    pub subcats: Option<usize>,
}
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct PageImageItem {
    pub original: Option<Original>,
    pub pageimage: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Original {
    // for PageImageItem only
    pub source: String,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct PageInfoItem {
    pub contentmodel: Option<String>,
    pub length: Option<usize>,
}
