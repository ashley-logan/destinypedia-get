use crate::NAMESPACE;
use crate::request::helpers::{ContinueStruct, ListString};
use derive_more::Display;
use serde::Serialize;
use serde_with_macros::{serde_as, skip_serializing_none};

/// This trait describes an mediaWiki Api 'action' (e.g. query, parse, opensearch)
/// The type implemnting this trait contains all action-specific parameters for that action
/// The trait extends Default and serde::Serialize
pub trait Action: Default + Serialize {}

impl Action for Query {}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Default)]
#[serde(tag = "action", rename_all = "lowercase", rename = "query")]
pub struct Query {
    #[serde_as(as = "Option<ListString>")]
    pub titles: Option<Vec<String>>,
    #[serde_as(as = "Option<ListString>")]
    pub pageids: Option<Vec<u32>>,
    #[serde_as(as = "Option<ListString>")]
    pub prop: Option<Vec<Prop>>,
    #[serde(flatten)]
    pub generator: Option<Generator>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub indexpageids: bool,
    #[serde_as(as = "Option<ContinueStruct>")]
    #[serde(flatten)]
    pub cont: Option<(String, String)>,
}

#[derive(Debug, Display)]
#[display(rename_all = "lowercase")]
enum ImageInfo {
    Timestamp,
    User,
    Userid,
    Comment,
    Parsedcomment,
    Canonicaltitle,
    Url,
    Size,
    Dimensions,
    SHA1,
    Mime,
    Mediatype,
    Metadata,
    Commonmetadata,
    Extmetadata,
}

#[derive(Debug, Display)]
#[display(rename_all = "lowercase")]
pub enum Prop {
    Info,
    PageImages,
    Images,
    ImageInfo,
    Categories,
    CategoryInfo,
}

#[derive(Debug, Display)]
#[display(rename_all = "lowercase")]
enum CategoryProp {
    Ids,
    Title,
    Sortkey,
    Sortkeyprefix,
    Type,
    Timestamp,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize)]
#[serde(tag = "generator", rename_all = "lowercase")]
pub enum Generator {
    AllImages {
        gaiprefix: Option<String>,
        gailimit: Limit,
    },
    AllPages {
        #[serde_as(as = "Option<ListString>")]
        gapnamespace: Option<Vec<NAMESPACE>>,
        gaplimit: Limit,
    },
    AllCategories {
        gacprefix: Option<String>,
        gacmin: Option<u32>,
        gacmax: Option<u32>,
        gaclimit: Limit,
    },
    CategoryMembers {
        #[serde(flatten)]
        identifier: GcmIdentifier,
        #[serde_as(as = "Option<ListString>")]
        gcmnamespace: Option<Vec<NAMESPACE>>,
        gcmlimit: Limit,
    },
    Random,
}
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum GcmIdentifier {
    GcmTitle(String),
    GcmPageid(u32),
}

/// Enum that represents a value of any limit parameter
/// Per mediaWiki Api, can be either a number between [0, 500] or "max"
/// Any number larger than 500 will be serialized as "max"
/// Defaults to 50
#[derive(Debug, Copy, Clone)]
pub enum Limit {
    Num(u16),
    Max,
}

impl Serialize for Limit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Limit::Max => serializer.serialize_str("max"),
            Limit::Num(n) if *n <= 500 => serializer.serialize_str((*n).to_string().as_str()),
            _ => serializer.serialize_str("max"),
        }
    }
}

impl Default for Limit {
    fn default() -> Self {
        Limit::Num(20_u16)
    }
}

impl Generator {
    pub fn allimages_with(gaiprefix: Option<String>, gailimit: Option<Limit>) -> Self {
        Generator::AllImages {
            gaiprefix,
            gailimit: gailimit.unwrap_or_default(),
        }
    }

    pub fn allpages_with(
        gapnamespace: Option<impl IntoIterator<Item = NAMESPACE>>,
        gaplimit: Option<Limit>,
    ) -> Self {
        Generator::AllPages {
            gapnamespace: gapnamespace.map(|v| v.into_iter().collect::<Vec<NAMESPACE>>()),
            gaplimit: gaplimit.unwrap_or_default(),
        }
    }

    pub fn allcategories_with(
        gacprefix: Option<String>,
        gacmin: Option<u32>,
        gacmax: Option<u32>,
        gaclimit: Option<Limit>,
    ) -> Self {
        Generator::AllCategories {
            gacprefix,
            gacmin,
            gacmax,
            gaclimit: gaclimit.unwrap_or_default(),
        }
    }

    pub fn categorymembers_with(
        identifier: GcmIdentifier,
        gcmnamespace: Option<Vec<NAMESPACE>>,
        gcmlimit: Option<Limit>,
    ) -> Self {
        Generator::CategoryMembers {
            identifier,
            gcmnamespace,
            gcmlimit: gcmlimit.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{Token, assert_ser_tokens};

    #[test]
    fn test_query_success() {
        let q = Query {
            titles: Some(vec!["simpleTitle".into(), "anotherOne".into()]),
            pageids: None,
            prop: Some(vec![Prop::Info, Prop::CategoryInfo, Prop::ImageInfo]),
            generator: None,
            indexpageids: false,
            cont: None,
        };

        assert_ser_tokens(
            &q,
            &[
                Token::Map { len: None },
                Token::Str("action"),
                Token::Str("query"),
                Token::Str("titles"),
                Token::Some,
                Token::Str("simpleTitle|anotherOne"),
                Token::Str("prop"),
                Token::Some,
                Token::Str("info|categoryinfo|imageinfo"),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn test_generator_success() {
        let g = Generator::CategoryMembers {
            identifier: GcmIdentifier::GcmTitle("Category:Test".into()),
            gcmnamespace: Some(vec![NAMESPACE::CATEGORY, NAMESPACE::PAGE]),
            gcmlimit: Limit::Max,
        };

        assert_ser_tokens(
            &g,
            &[
                Token::Map { len: None },
                Token::Str("generator"),
                Token::Str("categorymembers"),
                Token::Str("gcmtitle"),
                Token::Str("Category:Test"),
                Token::Str("gcmnamespace"),
                Token::Some,
                Token::Str("14|0"),
                Token::Str("gcmlimit"),
                Token::Str("max"),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn test_generator_constructor() {
        let g = Generator::allimages_with(None, Some(Limit::Num(20)));
        assert_ser_tokens(
            &g,
            &[
                Token::Struct {
                    name: "Generator",
                    len: 2,
                },
                Token::Str("generator"),
                Token::Str("allimages"),
                Token::Str("gailimit"),
                Token::Str("20"),
                Token::StructEnd,
            ],
        );
    }
}
