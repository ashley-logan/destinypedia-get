use crate::models::deserialize::items;
use serde::Deserialize;

pub trait PropResults {}

macro_rules! impl_prop_results {
    ($ty:ty) => {
        impl PropResults for $ty {}
    };
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ImageInfo {
    pub imageinfo: Vec<items::ImageInfoItem>,
}

impl_prop_results!(ImageInfo);

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct CategoryInfo {
    pub categoryinfo: items::CatgeoryInfoItem,
}

impl_prop_results!(CategoryInfo);

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Categories {
    pub categories: Vec<items::CategoryItem>,
}

impl_prop_results!(Categories);

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct PageImages {
    #[serde(flatten)]
    pub pageimages: items::PageImageItem,
}

impl_prop_results!(PageImages);

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Images {
    pub images: Vec<items::ImageItem>,
}

impl_prop_results!(Images);

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct PageInfo {
    #[serde(flatten)]
    pub pageinfo: items::PageInfoItem,
}

impl_prop_results!(PageInfo);

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

        let exp: ImageInfo = ImageInfo {
            imageinfo: vec![items::ImageInfoItem {
                canonicaltitle: Some("File:Clash of the Hive Gods.jpg".into()),
                size: Some(523694),
                width: Some(3840),
                height: Some(2160),
                url: Some(
                    "https://destiny.wiki.gallery/images/f/f7/Clash_of_the_Hive_Gods.jpg".into(),
                ),
                timestamp: None,
            }],
        };

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

        let exp: CategoryInfo = CategoryInfo {
            categoryinfo: items::CatgeoryInfoItem {
                size: Some(2966),
                pages: Some(2961),
                files: Some(3),
                subcats: Some(2),
            },
        };

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

        let exp = Categories {
            categories: vec![
                items::CategoryItem {
                    ns: 14,
                    title: "Category:Articles needing cleanup".into(),
                },
                items::CategoryItem {
                    ns: 14,
                    title: "Category:Articles needing fact cleanup".into(),
                },
                items::CategoryItem {
                    ns: 14,
                    title: "Category:Articles under construction".into(),
                },
            ],
        };

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

        let og = items::Original {
            source: "https://destiny.wiki.gallery/images/b/b4/Grimoire_The_Hive.jpg".into(),
            width: 560,
            height: 728,
        };

        let exp = PageImages {
            pageimages: items::PageImageItem {
                original: Some(og),
                pageimage: Some("Grimoire_The_Hive.jpg".into()),
            },
        };

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

        let exp = Images {
            images: vec![
                items::ImageItem {
                    ns: 6,
                    title: "File:Alakhul.jpg".into(),
                },
                items::ImageItem {
                    ns: 6,
                    title: "File:ArcS.png".into(),
                },
                items::ImageItem {
                    ns: 6,
                    title: "File:Battle on Saturn.jpg".into(),
                },
            ],
        };

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

        let exp = PageInfo {
            pageinfo: items::PageInfoItem {
                contentmodel: Some("wikitext".into()),
                length: Some(220921),
            },
        };

        assert_eq!(
            from_value::<PageInfo>(control).expect("Failed to convert control to PageInfo"),
            exp
        )
    }
}
