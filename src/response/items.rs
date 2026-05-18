use serde::Deserialize;

pub trait PropResults {
    type ItemType;
    fn empty(&self) -> bool;
    fn into_items(self) -> Vec<Self::ItemType>;
}

pub trait Item {
    fn is_empty(&self) -> bool;
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct Categories(Vec<CategoryItem>);

impl PropResults for Categories {
    type ItemType = CategoryItem;
    fn empty(&self) -> bool {
        self.0.iter().all(CategoryItem::is_empty)
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
pub struct CategoryInfo(pub CatgeoryInfoItem);

impl PropResults for CategoryInfo {
    type ItemType = CatgeoryInfoItem;
    fn empty(&self) -> bool {
        CatgeoryInfoItem::is_empty(&self.0)
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
pub struct Images(Vec<ImageItem>);

impl PropResults for Images {
    type ItemType = ImageItem;
    fn empty(&self) -> bool {
        self.0.iter().all(ImageItem::is_empty)
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
pub struct ImageInfo(Vec<ImageInfoItem>);

impl PropResults for ImageInfo {
    type ItemType = ImageInfoItem;
    fn empty(&self) -> bool {
        self.0.iter().all(ImageInfoItem::is_empty)
    }

    fn into_items(self) -> Vec<Self::ItemType> {
        self.0
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct ImageInfoItem {
    pub canonicaltitle: Option<String>,
    pub size: Option<u32>,
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
pub struct PageImages(pub PageImageItem);

impl PropResults for PageImages {
    type ItemType = PageImageItem;
    fn empty(&self) -> bool {
        PageImageItem::is_empty(&self.0)
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
pub struct Info(pub InfoItem);

impl PropResults for Info {
    type ItemType = InfoItem;
    fn empty(&self) -> bool {
        InfoItem::is_empty(&self.0)
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_value, json};

    #[test]
    fn test_image_info() {
        let control = json!({
            "imageinfo": [
                    {
                        "size": 523694,
                        "width": 3840,
                        "height": 2160,
                        "canonicaltitle": "File:Clash of the Hive Gods.jpg",
                        "url": "https://destiny.wiki.gallery/images/f/f7/Clash_of_the_Hive_Gods.jpg",
                        "descriptionurl": "https://www.destinypedia.com/File:Clash_of_the_Hive_Gods.jpg",
                        "descriptionshorturl": "https://www.destinypedia.com/index.php?curid=39300"
                    }
                ]
        });

        let exp: ImageInfo = ImageInfo(vec![ImageInfoItem {
            canonicaltitle: Some("File:Clash of the Hive Gods.jpg".into()),
            size: Some(523694),
            width: Some(3840),
            height: Some(2160),
            url: Some("https://destiny.wiki.gallery/images/f/f7/Clash_of_the_Hive_Gods.jpg".into()),
            timestamp: None,
        }]);

        assert_eq!(
            from_value::<ImageInfo>(control).expect("Failed to convert control to ImageInfo"),
            exp
        )
    }

    #[test]
    fn test_category_info() {
        let control = json!({
            "categoryinfo": {
                    "size": 2966,
                    "pages": 2961,
                    "files": 3,
                    "subcats": 2
                }
        });

        let exp: CategoryInfo = CategoryInfo(CatgeoryInfoItem {
            size: Some(2966),
            pages: Some(2961),
            files: Some(3),
            subcats: Some(2),
        });

        assert_eq!(
            from_value::<CategoryInfo>(control).expect("Failed to convert control to CategoryInfo"),
            exp
        )
    }

    #[test]
    fn test_categories() {
        let control = json!({
            "categories": [
                    {
                        "ns": 14,
                        "title": "Category:Articles needing cleanup"
                    },
                    {
                        "ns": 14,
                        "title": "Category:Articles needing fact cleanup"
                    },
                    {
                        "ns": 14,
                        "title": "Category:Articles under construction"
                    }
            ]
        });

        let exp = Categories(vec![
            CategoryItem {
                ns: 14,
                title: "Category:Articles needing cleanup".into(),
            },
            CategoryItem {
                ns: 14,
                title: "Category:Articles needing fact cleanup".into(),
            },
            CategoryItem {
                ns: 14,
                title: "Category:Articles under construction".into(),
            },
        ]);

        assert_eq!(
            from_value::<Categories>(control).expect("Failed to convert control to Categories"),
            exp
        )
    }

    #[test]
    fn test_page_images() {
        let control = json!({
            "original": {
                    "source": "https://destiny.wiki.gallery/images/b/b4/Grimoire_The_Hive.jpg",
                    "width": 560,
                    "height": 728
                },
                "pageimage": "Grimoire_The_Hive.jpg"
        });

        let og = Original {
            source: "https://destiny.wiki.gallery/images/b/b4/Grimoire_The_Hive.jpg".into(),
            width: 560,
            height: 728,
        };

        let exp = PageImages(PageImageItem {
            original: Some(og),
            pageimage: Some("Grimoire_The_Hive.jpg".into()),
        });

        assert_eq!(
            from_value::<PageImages>(control).expect("Failed to convert control to PageImages"),
            exp
        )
    }

    #[test]
    fn test_images() {
        let control = json!({
            "images": [
                    {
                        "ns": 6,
                        "title": "File:Alakhul.jpg"
                    },
                    {
                        "ns": 6,
                        "title": "File:ArcS.png"
                    },
                    {
                        "ns": 6,
                        "title": "File:Battle on Saturn.jpg"
                    }
            ]
        });

        let exp = Images(vec![
            ImageItem {
                ns: 6,
                title: "File:Alakhul.jpg".into(),
            },
            ImageItem {
                ns: 6,
                title: "File:ArcS.png".into(),
            },
            ImageItem {
                ns: 6,
                title: "File:Battle on Saturn.jpg".into(),
            },
        ]);

        assert_eq!(
            from_value::<Images>(control).expect("Failed to convert control to Images"),
            exp
        )
    }

    #[test]
    fn test_page_info() {
        let control = json!({
            "contentmodel": "wikitext",
            "pagelanguage": "en",
            "pagelanguagehtmlcode": "en",
            "pagelanguagedir": "ltr",
            "touched": "2026-04-12T04:03:19Z",
            "lastrevid": 390101,
            "length": 220921
        });

        let exp = Info(InfoItem {
            contentmodel: Some("wikitext".into()),
            length: Some(220921),
        });

        assert_eq!(
            from_value::<Info>(control).expect("Failed to convert control to PageInfo"),
            exp
        )
    }
}
