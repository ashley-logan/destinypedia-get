use serde::ser::SerializeMap;
use serde_with::SerializeAs;
use std::fmt::Display;

/// This struct should never be used directly and only exists as
/// an interface for converting mulit-item strings to proper
/// mediaWiki format ('|' = seperator) via SerializeAs
pub(crate) struct ListString;

impl<T: Display> SerializeAs<Vec<T>> for ListString {
    fn serialize_as<S>(source: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v: Vec<String> = source.into_iter().map(|x| x.to_string()).collect();

        serializer.serialize_str(v.join(r"|").as_str())
    }
}

impl SerializeAs<Vec<crate::NAMESPACE>> for ListString {
    fn serialize_as<S>(source: &Vec<crate::NAMESPACE>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v: Vec<String> = source.iter().map(|ns| ns.clone().into()).collect();

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

pub(crate) struct ContinueStruct;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NAMESPACE;
    use crate::request::Prop;
    use serde::Serialize;
    use serde_test::{Token, assert_ser_tokens};
    use serde_with_macros::serde_as;

    #[serde_as]
    #[derive(Debug, Serialize, PartialEq)]
    struct TestStruct<T>
    where
        ListString: SerializeAs<Vec<T>>,
    {
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
