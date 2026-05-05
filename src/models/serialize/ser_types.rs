use std::fmt::Display;

use super::NAMESPACE;
use super::query::Prop;
use serde_with::ser::SerializeAs;

impl<T: Display> SerializeAs<Vec<T>> for ListString {
    fn serialize_as<S>(source: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let arr: Vec<String> = source.iter().map(|x| x.to_string()).collect();

        let s: String = arr.join(r"|");

        serializer.serialize_str(s.as_str())
    }
}

pub struct ListString;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::serialize::{NAMESPACE, query::Prop};
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
