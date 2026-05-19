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
    pub ns: crate::NAMESPACE,
    pub title: String,
}

impl Item for CategoryItem {
    fn is_empty(&self) -> bool {
        false
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct CategoryInfo(pub CategoryInfoItem);

impl PropResults for CategoryInfo {
    type ItemType = CategoryInfoItem;
    fn empty(&self) -> bool {
        CategoryInfoItem::is_empty(&self.0)
    }

    fn into_items(self) -> Vec<Self::ItemType> {
        vec![self.0]
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct CategoryInfoItem {
    pub size: Option<u32>,
    pub pages: Option<u32>,
    pub files: Option<u32>,
    pub subcats: Option<u32>,
}

impl Item for CategoryInfoItem {
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
    use serde_test::{assert_de_tokens, Token};

    #[test]
    fn test_image_info() {
    
        let json = json!([
                    {
                        "size": 523694,
                        "width": 3840,
                        "height": 2160,
                        "canonicaltitle": "File:Clash of the Hive Gods.jpg",
                        "url": "https://destiny.wiki.gallery/images/f/f7/Clash_of_the_Hive_Gods.jpg",
                        "descriptionurl": "https://www.destinypedia.com/File:Clash_of_the_Hive_Gods.jpg",
                        "descriptionshorturl": "https://www.destinypedia.com/index.php?curid=39300"
                    }
                ]);

        let resp: ImageInfo = from_value(json).expect("failed to convert json into ImageInfo");

        // let exp: ImageInfo = ImageInfo(vec![ImageInfoItem {
        //     canonicaltitle: Some("File:Clash of the Hive Gods.jpg".into()),
        //     size: Some(523694),
        //     width: Some(3840),
        //     height: Some(2160),
        //     url: Some("https://destiny.wiki.gallery/images/f/f7/Clash_of_the_Hive_Gods.jpg".into()),
        //     timestamp: None,
        // }]);

        assert_de_tokens(&resp, &[
            Token::Seq { len: Some(1) },
            Token::Struct { name: "ImageInfoItem", len: 6 },
            Token::Str("canonicaltitle"),
            Token::Some,
            Token::Str("File:Clash of the Hive Gods.jpg"),
            Token::Str("size"),
            Token::Some,
            Token::U32(523694),
            Token::Str("width"),
            Token::Some,
            Token::U32(3840),
            Token::Str("height"),
            Token::Some,
            Token::U32(2160),
            Token::Str("url"),
            Token::Some,
            Token::Str("https://destiny.wiki.gallery/images/f/f7/Clash_of_the_Hive_Gods.jpg"),
            Token::Str("timestamp"),
            Token::None,
            Token::StructEnd,
            Token::SeqEnd,
        ]);

        
    }

    #[test]
    fn test_category_info() {
        let control = json!({
                    "size": 2966,
                    "pages": 2961,
                    "files": 3,
                    "subcats": 2
        });

        let resp: CategoryInfo = from_value(control).expect("failed to convert json to CategoryInfo");

        assert_de_tokens(
            &resp, 
        &[
                Token::TupleStruct { name: "CategoryInfo", len: 1 },
                Token::Struct { name: "CategoryInfoItem", len: 4 },
                Token::Str("size"),
                Token::Some,
                Token::U32(2966),
                Token::Str("pages"),
                Token::Some,
                Token::U32(2961),
                Token::Str("files"),
                Token::Some,
                Token::U32(3),
                Token::Str("subcats"),
                Token::Some,
                Token::U32(2),
                Token::StructEnd,
                Token::TupleStructEnd
        ]);
        
        // let exp: CategoryInfo = CategoryInfo(CategoryInfoItem {
        //     size: Some(2966),
        //     pages: Some(2961),
        //     files: Some(3),
        //     subcats: Some(2),
        // });
    }

    #[test]
    fn test_categories() {
        let control = json!(
            [
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
        );

        let resp: Categories = from_value(control).expect("Failed to convert json to categories");

        assert_de_tokens(
            &resp,
            &[
                Token::Seq { len: Some(3) },
                Token::Struct { name: "CategoryItem", len: 2 },
                Token::Str("ns"),
                Token::U16(14),
                Token::Str("title"),
                Token::Str("Category:Articles needing cleanup"),
                Token::StructEnd,
                Token::Struct { name: "CategoryItem", len: 2 },
                Token::Str("ns"),
                Token::U16(14),
                Token::Str("title"),
                Token::Str("Category:Articles needing fact cleanup"),
                Token::StructEnd,
                Token::Struct { name: "CategoryItem", len: 2 },
                Token::Str("ns"),
                Token::U16(14),
                Token::Str("title"),
                Token::Str("Category:Articles under construction"),
                Token::StructEnd,
                Token::SeqEnd
            ]
        );

        // let exp = Categories(vec![
        //     CategoryItem {
        //         ns: 14,
        //         title: "Category:Articles needing cleanup".into(),
        //     },
        //     CategoryItem {
        //         ns: 14,
        //         title: "Category:Articles needing fact cleanup".into(),
        //     },
        //     CategoryItem {
        //         ns: 14,
        //         title: "Category:Articles under construction".into(),
        //     },
        // ]);

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

        let resp: PageImages = from_value(control).expect("Failed to convert json to PageImages");

        assert_de_tokens(
        &resp,
        &[
            // Pretend the transparent wrapper is a struct
            Token::Struct { name: "PageImageItem", len: 2 },

            // --- original ---
            Token::Str("original"),
            Token::Some,
            Token::Struct { name: "Original", len: 3 },

            Token::Str("source"),
            Token::Str("https://destiny.wiki.gallery/images/b/b4/Grimoire_The_Hive.jpg"),

            Token::Str("width"),
            Token::U32(560),

            Token::Str("height"),
            Token::U32(728),

            Token::StructEnd,

            // --- pageimage ---
            Token::Str("pageimage"),
            Token::Some,
            Token::Str("Grimoire_The_Hive.jpg"),

            Token::StructEnd,
        ],
    );

    }

    #[test]
    fn test_images() {
        let control = json!( [
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
            ]);
        let resp: Images = from_value(control).expect("Failed to convert json to Images");

        assert_de_tokens(
        &resp,
        &[
            Token::Seq { len: Some(3) },

            // 1st item
            Token::Map { len: Some(2) },
            Token::Str("ns"),
            Token::U32(6),
            Token::Str("title"),
            Token::Str("File:Alakhul.jpg"),
            Token::MapEnd,

            // 2nd item
            Token::Map { len: Some(2) },
            Token::Str("ns"),
            Token::U32(6),
            Token::Str("title"),
            Token::Str("File:ArcS.png"),
            Token::MapEnd,

            // 3rd item
            Token::Map { len: Some(2) },
            Token::Str("ns"),
            Token::U32(6),
            Token::Str("title"),
            Token::Str("File:Battle on Saturn.jpg"),
            Token::MapEnd,

            Token::SeqEnd,
        ],
    );

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

        let resp: Info = from_value(control).expect("Failed to convert json to Info");

        assert_de_tokens(
        &resp,
        &[
            // Outer newtype struct: Info(...)
            Token::Struct { name: "InfoItem", len: 2 },

            Token::Str("contentmodel"),
            Token::Some,
            Token::Str("wikitext"),

            Token::Str("length"),
            Token::Some,
            Token::U64(220921),

            Token::StructEnd,
        ],
    );

    }
}
