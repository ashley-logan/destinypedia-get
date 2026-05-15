use super::ser_types::{ContinueStruct, Limit, ListString};
use super::{NAMESPACE, Prop};
use derive_more::Display;
use serde::Serialize;
use serde_with;
use serde_with_macros::{serde_as, skip_serializing_none};

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, derive_more::PartialEq, derive_more::Eq, Default)]
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

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, derive_more::PartialEq, derive_more::Eq)]
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
#[derive(Debug, Serialize, derive_more::PartialEq, derive_more::Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum GcmIdentifier {
    GcmTitle(String),
    GcmPageid(u32),
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

    // use linked_hash_map::LinkedHashMap;
    // use serde_test::{Token, assert_tokens};

    // #[test]
    // fn test_ser_de_empty() {
    //     let map = LinkedHashMap::<char, u32>::new();

    //     assert_tokens(&map, &[
    //         Token::Map { len: Some(0) },
    //         Token::MapEnd,
    //     ]);
    // }

    // #[test]
    // fn test_ser_de() {
    //     let mut map = LinkedHashMap::new();
    //     map.insert('b', 20);
    //     map.insert('a', 10);
    //     map.insert('c', 30);

    //     assert_tokens(&map, &[
    //         Token::Map { len: Some(3) },
    //         Token::Char('b'),
    //         Token::I32(20),

    //         Token::Char('a'),
    //         Token::I32(10),

    //         Token::Char('c'),
    //         Token::I32(30),
    //         Token::MapEnd,
    //     ]);
    // }
}
