use super::query::{Generator, Prop, Query};
use crate::{Error, Result};
use derive_more::Display;
use serde::Serialize;
use serde_with_macros::{apply, skip_serializing_none};

#[derive(Debug, Serialize, PartialEq, Eq, Display)]
#[serde(rename_all = "lowercase")]
pub enum ErrorFormat {
    PlainText,
    WikiText,
    HTML,
    Raw,
    None,
    BC,
}

#[derive(Debug, Serialize, PartialEq, Eq, Display, Default)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    #[default]
    Json,
    JsonFm,
    None,
    Php,
    PhpFm,
    RawFm,
    Xml,
    XmlFm,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct PARAMS<T> {
    #[serde(flatten)]
    pub params: T,
    #[serde(flatten)]
    pub continue_: Option<(String, String)>,
    pub format: Format,
}

// titles: None,
//                 pageids: None,
//                 prop: None,
//                 list: None,
// impl PARAMS< {
//     pub fn new_query() -> Self {
//         Self::Query {
//             titles: None,
//             pageids: None,
//             prop: None,
//             generator: None,
//             cont: None,
//         }
//     }

//     pub fn titles<I, T>(mut self, titles_: I) -> Self
//     where
//         I: IntoIterator<Item = T>,
//         T: AsRef<str>,
//     {
//         let titles_: Vec<String> = titles_.into_iter().collect();

//         match &mut self {
//             Self::Query {
//                 titles: Some(t), ..
//             } => {
//                 t.append(&mut titles_);
//             }
//             Self::Query { titles: None, .. } => {
//                 self.titles = titles_;
//             }
//         }
//     }
// }

impl PARAMS<Query> {
    pub fn new() -> Self {
        Self {
            params: Query::default(),
            continue_: None,
            format: Format::default(),
        }
    }

    pub fn with_titles<I, T>(mut self, titles_: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: ToString,
    {
        let new_titles: Vec<String> = titles_.into_iter().map(|x| x.to_string()).collect();

        if !new_titles.is_empty() {
            self.params.titles = Some(new_titles);
        } else {
            self.params.titles = None;
        }
        self
    }

    pub fn with_pageids<I, T>(mut self, pageids_: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<usize>,
    {
        let new_ids: Vec<usize> = pageids_.into_iter().map(T::into).collect();

        if !new_ids.is_empty() {
            self.params.pageids = Some(new_ids);
        } else {
            self.params.pageids = None;
        }

        self
    }

    pub fn with_props<I, T>(mut self, props_: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Prop>,
    {
        let new_props: Vec<Prop> = props_.into_iter().map(T::into).collect();

        if !new_props.is_empty() {
            self.params.prop = Some(new_props);
        } else {
            self.params.prop = None;
        }

        self
    }

    pub fn with_generator(mut self, generator_: Generator) -> Self {
        self.params.generator = Some(generator_);

        self
    }

    pub fn with_continue<T: AsRef<str>>(mut self, ckey: T, cval: T) -> Self {
        self.continue_ = Some((ckey.as_ref().into(), cval.as_ref().into()));

        self
    }

    pub fn with_format(mut self, format_: Format) -> Self {
        self.format = format_;
        self
    }

    pub fn append_titles<I, T>(&mut self, titles_: I)
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        if let Some(v) = &mut self.params.titles {
            v.append(
                &mut titles_
                    .into_iter()
                    .map(|x| x.as_ref().to_string())
                    .collect(),
            );
        } else {
            self.params.titles = Some(
                titles_
                    .into_iter()
                    .map(|x| x.as_ref().to_string())
                    .collect(),
            );
        }
    }

    pub fn append_pageids<I, T>(&mut self, pageids_: I)
    where
        I: IntoIterator<Item = T>,
        T: Into<usize>,
    {
        if let Some(v) = &mut self.params.pageids {
            v.append(&mut pageids_.into_iter().map(|x| x.into()).collect());
        } else {
            self.params.pageids = Some(pageids_.into_iter().map(|x| x.into()).collect());
        }
    }

    pub fn set_continue_value<T: AsRef<str>>(&mut self, cval: T) -> Result<()> {
        if let Some(tup) = &mut self.continue_ {
            tup.1 = cval.as_ref().to_string();
            Ok(())
        } else {
            Err(Error::Params)
        }
    }

    pub fn set_continue_key<T: AsRef<str>>(&mut self, ckey: T) -> Result<()> {
        if let Some(tup) = &mut self.continue_ {
            tup.0 = ckey.as_ref().to_string();
            Ok(())
        } else {
            Err(Error::Params)
        }
    }

    pub fn set_continue<T: AsRef<str>>(&mut self, ckey: T, cval: T) {
        self.continue_ = Some((ckey.as_ref().into(), cval.as_ref().into()));
    }

    pub fn set_format(&mut self, format_: Format) {
        self.format = format_;
    }
}

// #[cfg(test)]
// #[allow(non_snake_case)]
// mod tests {
//     use super::*;
//     use query::*;
//     use serde_json::{json, to_value};

//     fn construct_unvalidated_but_valid_query_PARAMS() -> PARAMS<Query> {
//         let q = Query::new()
//             .pageids(&[100, 420, 8887, u16::MAX])
//             .prop(Prop::Info)
//             .validate()
//             .unwrap();
//         PARAMS::from_query(q)
//     }

//     #[test]
//     fn test_PARAMS_inplace_construction1() {
//         let control = construct_unvalidated_but_valid_query_PARAMS()
//             .format(Format::JsonFm)
//             .err_format(ErrorFormat::PlainText)
//             .validate()
//             .unwrap();

//         let mut exp: PARAMS<Query> = PARAMS::new();
//         exp.set_format(Format::JsonFm);
//         exp.set_err_format(ErrorFormat::PlainText);
//         exp.set_pageids(&[100, 420, 8887, u16::MAX]);
//         exp.set_prop(Prop::Info);
//         exp = exp.validate().unwrap();

//         assert_eq!(control, exp)
//     }

//     #[test]
//     fn test_PARAMS_chained_construction1() {
//         let control = construct_unvalidated_but_valid_query_PARAMS()
//             .validate()
//             .unwrap();

//         let exp: PARAMS<Query> = PARAMS::new()
//             .pageids(&[100, 420, 8887, u16::MAX])
//             .prop(Prop::Info)
//             .format(Format::Json)
//             .err_format(ErrorFormat::Raw)
//             .validate()
//             .unwrap();

//         assert_eq!(control, exp)
//     }

//     #[test]
//     fn test_PARAMS_validatation_success1() {
//         let url = construct_unvalidated_but_valid_query_PARAMS();
//         assert!(url.validate().is_ok())
//     }

//     #[test]
//     fn test_PARAMS_validatation_fail1() {
//         let url = construct_unvalidated_but_valid_query_PARAMS().pageids(&vec![]);
//         assert!(url.validate().is_err())
//     }
//     #[test]
//     fn test_PARAMS_serialize_success1() {
//         let url = construct_unvalidated_but_valid_query_PARAMS();

//         assert!(to_value(url).is_ok())
//     }

//     #[test]
//     fn test_PARAMS_serialize_success2() {
//         let val = to_value(construct_unvalidated_but_valid_query_PARAMS()).unwrap();

//         let exp = json!({
//             "action": "query",
//             "pageids": [100, 420, 8887, 65535],
//             "prop": "info",
//             "format": "json",
//             "errorformat": "raw"
//         });

//         assert_eq!(val, exp);
//     }
// }
