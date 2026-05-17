use serde::Deserialize;

pub trait PropResults {
    type ItemType;
    fn empty(&self) -> bool;
    fn iter_items(&self) -> std::slice::Iter<'_, Self::ItemType>;
    fn into_items(self) -> Vec<Self::ItemType>;
}

pub trait Item {
    fn is_empty(&self) -> bool;
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct CategoriesProp(Vec<CategoryItem>);

impl PropResults for CategoriesProp {
    type ItemType = CategoryItem;
    fn empty(&self) -> bool {
        self.0.iter().all(CategoryItem::is_empty)
    }
    fn iter_items(&self) -> std::slice::Iter<'_, Self::ItemType> {
        self.0.iter()
    }
    fn into_items(self) -> Vec<Self::ItemType> {
        self.0
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
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
#[serde(transparent)]
pub struct CategoryInfoProp(CatgeoryInfoItem);

impl PropResults for CategoryInfoProp {
    type ItemType = CatgeoryInfoItem;
    fn empty(&self) -> bool {
        CatgeoryInfoItem::is_empty(&self.0)
    }
    fn iter_items(&self) -> std::slice::Iter<'_, Self::ItemType> {
        [self.0].iter()
    }
    fn into_items(self) -> Vec<Self::ItemType> {
        vec![self.0]
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct CatgeoryInfoItem {
    pub size: Option<u32>,
    pub pages: Option<u32>,
    pub files: Option<u32>,
    pub subcats: Option<u32>,
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
#[serde(transparent)]
pub struct ImagesProp(Vec<ImageItem>);

impl PropResults for ImagesProp {
    type ItemType = ImageItem;
    fn empty(&self) -> bool {
        self.0.iter().all(ImageItem::is_empty)
    }
    fn iter_items(&self) -> std::slice::Iter<'_, Self::ItemType> {
        self.0.iter()
    }
    fn into_items(self) -> Vec<Self::ItemType> {
        self.0
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
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
#[serde(transparent)]
pub struct ImageInfoProp(Vec<ImageInfoItem>);

impl PropResults for ImageInfoProp {
    type ItemType = ImageInfoItem;
    fn empty(&self) -> bool {
        self.0.iter().all(ImageInfoItem::is_empty)
    }
    fn iter_items(&self) -> std::slice::Iter<'_, Self::ItemType> {
        self.0.iter()
    }
    fn into_items(self) -> Vec<Self::ItemType> {
        self.0
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct ImageInfoItem {
    pub canonicaltitle: Option<String>,
    pub size: Option<u128>,
    pub width: Option<u32>,
    pub height: Option<u32>,
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
#[serde(transparent)]
pub struct PageImagesProp(PageImageItem);

impl PropResults for PageImagesProp {
    type ItemType = PageImageItem;
    fn empty(&self) -> bool {
        PageImageItem::is_empty(&self.0)
    }
    fn iter_items(&self) -> std::slice::Iter<'_, Self::ItemType> {
        [self.0].iter()
    }
    fn into_items(self) -> Vec<Self::ItemType> {
        vec![self.0]
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
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

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Original {
    // for PageImageItem only
    pub source: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct InfoProp(InfoItem);

impl PropResults for InfoProp {
    type ItemType = InfoItem;
    fn empty(&self) -> bool {
        InfoItem::is_empty(&self.0)
    }
    fn iter_items(&self) -> std::slice::Iter<'_, Self::ItemType> {
        [self.0].iter()
    }
    fn into_items(self) -> Vec<Self::ItemType> {
        vec![self.0]
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct InfoItem {
    pub contentmodel: Option<String>,
    pub length: Option<u32>,
}

impl Item for InfoItem {
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
