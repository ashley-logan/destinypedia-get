use serde::{Serialize, ser::SerializeMap};
use serde_with::ser::SerializeAs;
use std::fmt::Display;

/// This trait describes an mediaWiki Api 'action' (e.g. query, parse, opensearch)
/// The type implemnting this trait contains all action-specific parameters for that action
/// The trait extends Default and serde::Serialize
pub trait Action: Default + Serialize {}

impl Action for super::query::Query {}

/// This struct should never be used directly and only exists as
/// an interface for converting mulit-item strings to proper
/// mediaWiki format ('|' = seperator) via SerializeAs
pub(super) struct ListString;

pub struct ContinueStruct;

impl<T: Display> SerializeAs<Vec<T>> for ListString {
    fn serialize_as<S>(source: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v: Vec<String> = source.into_iter().map(|x| x.to_string()).collect();

        serializer.serialize_str(v.join(r"|").as_str())
    }
}

impl<T: Display> SerializeAs<[T]> for ListString {
    fn serialize_as<S>(source: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v: Vec<String> = source.iter().map(|x| x.to_string()).collect();

        serializer.serialize_str(v.join(r"|").as_str())
    }
}

impl<T: AsRef<str>> SerializeAs<(T, T)> for ContinueStruct {
    fn serialize_as<S>(source: &(T, T), serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(source.0.as_ref(), source.1.as_ref())?;

        map.end()
    }
}

/// Enum that represents a value of any limit parameter
/// Per mediaWiki Api, can be either a number between [0, 500] or "max"
/// Any number larger than 500 will be serialized as "max"
/// Defaults to 50
#[derive(Debug, Copy, Clone, derive_more::Eq, derive_more::PartialEq)]
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
        Limit::Num(50_u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::serialize::{NAMESPACE, Prop};
    use serde::Serialize;
    use serde_test::{Token, assert_ser_tokens};
    use serde_with_macros::serde_as;

    // let s = S { a: 0, b: 0 };
    // assert_ser_tokens(
    //     &s,
    //     &[
    //         Token::Struct { name: "S", len: 2 },
    //         Token::Str("a"),
    //         Token::U8(0),
    //         Token::Str("b"),
    //         Token::U8(0),
    //         Token::StructEnd,
    //     ],
    // );

    #[serde_as]
    #[derive(Debug, Serialize, PartialEq)]
    struct TestStruct<T: Display> {
        #[serde_as(as = "Option<ListString>")]
        testval: Option<Vec<T>>,
    }

    #[test]
    fn test_liststring_adapter_success1() {
        let tester: TestStruct<String> = TestStruct {
            testval: Some(vec!["oneprop".into(), "twoprop".into()]),
        };

        assert_ser_tokens(
            &tester,
            &[
                Token::Struct {
                    name: "TestStruct",
                    len: 1,
                },
                Token::Str("testval"),
                Token::Some,
                Token::Str("oneprop|twoprop"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn test_liststring_adapter_success2() {
        let tester: TestStruct<NAMESPACE> = TestStruct {
            testval: Some(vec![
                NAMESPACE::PAGE,
                NAMESPACE::DESTINYPEDIA,
                NAMESPACE::FILE,
            ]),
        };

        assert_ser_tokens(
            &tester,
            &[
                Token::Struct {
                    name: "TestStruct",
                    len: 1,
                },
                Token::Str("testval"),
                Token::Some,
                Token::Str("0|4|6"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn test_liststring_adapter_success3() {
        let tester: TestStruct<Prop> = TestStruct {
            testval: Some(vec![Prop::Info, Prop::PageImages]),
        };

        assert_ser_tokens(
            &tester,
            &[
                Token::Struct {
                    name: "TestStruct",
                    len: 1,
                },
                Token::Str("testval"),
                Token::Some,
                Token::Str("info|pageimages"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn test_liststring_adapter_success4() {
        let tester: TestStruct<Prop> = TestStruct {
            testval: Some(vec![Prop::ImageInfo]),
        };

        assert_ser_tokens(
            &tester,
            &[
                Token::Struct {
                    name: "TestStruct",
                    len: 1,
                },
                Token::Str("testval"),
                Token::Some,
                Token::Str("imageinfo"),
                Token::StructEnd,
            ],
        );
    }
}
