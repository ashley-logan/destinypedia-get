use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ListResponse {
    #[serde(rename = "continue")]
    pub cont: Option<Continue>,
    pub query: ListItems,
}
#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Continue {
    #[serde(rename = "continue")]
    contin: String,
    #[serde(flatten)]
    sub_cont: SubContinue,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum SubContinue {
    ApContinue(String),
    AiContinue(String),
    AcContinue(String),
    CmContinue(String),
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum ListItems {
    Allimages(Vec<ImageItem>),
    Allcategories(Vec<CategoryItem>),
    Allpages(Vec<PageItem>),
    CategoryMembers(Vec<CmItem>),
}

pub trait Item<'de>: Deserialize<'de> + PartialEq + Eq {}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct ImageItem {
    name: String,
    timestamp: String,
    url: String,
    descriptionurl: String,
    descriptionshorturl: String,
    ns: u32,
    title: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct CategoryItem {
    category: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct PageItem {
    pageid: u32,
    ns: u32,
    title: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct CmItem {
    pageid: u32,
    ns: u32,
    title: String,
    #[serde(rename = "type")]
    type_: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_value, json, to_string_pretty};

    fn get_serialized() -> serde_json::Value {
        json!({
            "continue": {
                "aicontinue": "((STEEL_MEDULLA~)).jpg",
                "continue": "-||"
            },
            "query": {
                // "pages": {
                //     "14441": {
                //         "pageid": 14441,
                //         "ns": 0,
                //         "title": "Dreaming City"
                //     }
                // },
                "allimages": [
                    {
                        "name": "'Act_on_Instinct'.png",
                        "timestamp": "2024-07-02T13:45:06Z",
                        "url": "https://destiny.wiki.gallery/images/7/76/%27Act_on_Instinct%27.png",
                        "descriptionurl": "https://www.destinypedia.com/File:%27Act_on_Instinct%27.png",
                        "descriptionshorturl": "https://www.destinypedia.com/index.php?curid=44354",
                        "ns": 6,
                        "title": "File:'Act on Instinct'.png"
                    },
                ]
            }
        })
    }

    fn get_structified() -> ListResponse {
        ListResponse {
            cont: Some(Continue {
                contin: "-||".to_string(),
                sub_cont: SubContinue::AiContinue("((STEEL_MEDULLA~)).jpg".to_string()),
            }),
            query: ListItems::Allimages(vec![ImageItem {
                name: "'Act_on_Instinct'.png".to_string(),
                timestamp: "2024-07-02T13:45:06Z".to_string(),
                url: "https://destiny.wiki.gallery/images/7/76/%27Act_on_Instinct%27.png"
                    .to_string(),
                descriptionurl: "https://www.destinypedia.com/File:%27Act_on_Instinct%27.png"
                    .to_string(),
                descriptionshorturl: "https://www.destinypedia.com/index.php?curid=44354"
                    .to_string(),
                ns: 6_u32,
                title: "File:'Act on Instinct'.png".to_string(),
            }]),
        }
    }

    // #[test]
    // fn printer() {
    //     let x = get_serialized();
    //     let s = to_string_pretty(&x).unwrap();
    //     print!("{}", s);
    //     assert!(true)
    // }
    // #[test]
    // fn deburg() {
    //     let x = get_structified();
    //     dbg!(x);
    //     assert!(true)
    // }

    #[test]
    fn test_deserialize_success1() {
        let y: Result<ListResponse, serde_json::Error> = from_value(get_serialized());
        assert!(y.is_ok())
    }

    #[test]
    fn test_deserialize_eq() {
        let x: ListResponse = get_structified();
        let y: ListResponse = from_value(get_serialized()).unwrap();

        assert_eq!(x, y)
    }
}
