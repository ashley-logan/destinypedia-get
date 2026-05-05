use super::NAMESPACE;
use super::ser_types::ListString;
use derive_more::Display;
use serde::Serialize;
use serde_with;
use serde_with_macros::{apply, serde_as, skip_serializing_none};

#[skip_serializing_none]
#[apply(
    Option<Vec<_>> => #[serde_as(as = "Option<ListString>")]
)]
#[derive(Debug, Serialize, PartialEq, Eq, Default)]
#[serde(tag = "action", rename_all = "lowercase")]
pub struct Query {
    pub titles: Option<Vec<String>>,
    pub pageids: Option<Vec<usize>>,
    pub prop: Option<Vec<Prop>>,
    #[serde(flatten)]
    pub generator: Option<Generator>,
    #[serde(flatten)]
    pub cont: Option<(String, String)>,
}

#[derive(Debug, Serialize, PartialEq, Eq, Display)]
#[serde(rename_all = "lowercase")]
#[display(rename_all = "lowercase")]
pub enum Prop {
    Info,
    PageImages,
    Images,
    ImageInfo,
    Categories,
    CategoryInfo,
    FileUsage,
}

serde_with::serde_conv!(
    #[doc = "Serialize and deserialize Category strings"]
    CategoryString,
    String,
    |s: &String| {
        if !s.starts_with("Category:") {
            format!("Category:{}", s)
        } else {
            s.to_string()
        }
    },
    |value: String| -> Result<_, std::convert::Infallible> { Ok(value) }
);

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "generator", rename_all = "lowercase")]
pub enum Generator {
    AllImages {
        #[serde_as(as = "Option<ListString>")]
        gaiprop: Option<Vec<Prop>>,
        gaiprefix: Option<String>,
        gailimit: Option<usize>,
    },
    AllPages {
        #[serde_as(as = "Option<ListString>")]
        gapnamespace: Option<Vec<NAMESPACE>>,
        gaplimit: Option<usize>,
    },
    AllCategories {
        gacprefix: Option<String>,
        gacmin: Option<usize>,
        gacmax: Option<usize>,
        gaclimit: Option<usize>,
    },
    CategoryMembers {
        gcmtitle: String,
        #[serde_as(as = "Option<ListString>")]
        gcmprop: Option<Vec<Prop>>,
        #[serde_as(as = "Option<ListString>")]
        gcmnamespace: Option<Vec<NAMESPACE>>,
    },
    Random,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{Token, assert_ser_tokens};

    #[test]
    fn test_generator_success() {
        let g = Generator::CategoryMembers {
            gcmtitle: "Category:Test".into(),
            gcmprop: Some(vec![Prop::Categories]),
            gcmnamespace: Some(vec![NAMESPACE::CATEGORY, NAMESPACE::PAGE]),
        };

        assert_ser_tokens(
            &g,
            &[
                Token::Struct {
                    name: "Generator",
                    len: 4,
                },
                Token::Str("generator"),
                Token::Str("categorymembers"),
                Token::Str("gcmtitle"),
                Token::Str("Category:Test"),
                Token::Str("gcmprop"),
                Token::Some,
                Token::Str("categories"),
                Token::Str("gcmnamespace"),
                Token::Some,
                Token::Str("14|0"),
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
            cont: None,
        };

        assert_ser_tokens(
            &q,
            &[
                Token::Map { len: None },
                Token::Str("action"),
                Token::Str("Query"),
                Token::Str("titles"),
                Token::Some,
                Token::Str("simpleTitle|anotherOne"),
                Token::Str("pageids"),
                Token::None,
                Token::Str("prop"),
                Token::Some,
                Token::Str("info|categoryinfo|imageinfo"),
                Token::Str("generator"),
                Token::None,
                Token::Str("cont"),
                Token::None,
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
