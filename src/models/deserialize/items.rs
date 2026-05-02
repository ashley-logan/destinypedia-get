use serde::Deserialize;

pub trait Item {
    fn is_empty(&self) -> bool;
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ImageItem {
    pub ns: u32,
    pub title: String,
}

impl Item for ImageItem {
    fn is_empty(&self) -> bool {
        false
    }
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

impl Item for ImageInfoItem {
    fn is_empty(&self) -> bool {
        if self.canonicaltitle.is_some() {
            false
        } else if self.size.is_some() {
            false
        } else if self.width.is_some() {
            false
        } else if self.height.is_some() {
            false
        } else if self.url.is_some() {
            false
        } else if self.timestamp.is_some() {
            false
        } else {
            true
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct CategoryItem {
    pub ns: u32,
    pub title: String,
}

impl Item for CategoryItem {
    fn is_empty(&self) -> bool {
        false
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct CatgeoryInfoItem {
    pub size: Option<usize>,
    pub pages: Option<usize>,
    pub files: Option<usize>,
    pub subcats: Option<usize>,
}

impl Item for CatgeoryInfoItem {
    fn is_empty(&self) -> bool {
        if self.size.is_some() {
            false
        } else if self.pages.is_some() {
            false
        } else if self.files.is_some() {
            false
        } else if self.subcats.is_some() {
            false
        } else {
            true
        }
    }
}
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct PageImageItem {
    pub original: Option<Original>,
    pub pageimage: Option<String>,
}

impl Item for PageImageItem {
    fn is_empty(&self) -> bool {
        if self.original.is_some() {
            false
        } else if self.pageimage.is_some() {
            false
        } else {
            true
        }
    }
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

impl Item for PageInfoItem {
    fn is_empty(&self) -> bool {
        if self.contentmodel.is_some() {
            false
        } else if self.length.is_some() {
            false
        } else {
            true
        }
    }
}
