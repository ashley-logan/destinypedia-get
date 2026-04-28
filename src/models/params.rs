use crate::{ActionType, Error, Result, parse, query};
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Query,
    Parse,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ErrorFormat {
    PlainText,
    WikiText,
    HTML,
    Raw,
    None,
    BC,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Json,
    JsonFm,
    None,
    Php,
    PhpFm,
    RawFm,
    Xml,
    XmlFm,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct PARAMS<T: ActionType> {
    action: Action,
    #[serde(flatten)]
    action_values: T,
    format: Format,
    errorformat: ErrorFormat,
    #[serde(skip)]
    validated: bool,
}

// titles: None,
//                 pageids: None,
//                 prop: None,
//                 list: None,
impl PARAMS<query::Query> {
    pub fn new() -> Self {
        Self {
            action: Action::Query,
            action_values: query::Query::new(),
            format: Format::Json,
            errorformat: ErrorFormat::Raw,
            validated: false,
        }
    }

    pub fn from_query(q: query::Query) -> Self {
        Self {
            action: Action::Query,
            action_values: q,
            format: Format::Json,
            errorformat: ErrorFormat::Raw,
            validated: false,
        }
    }

    pub fn titles<T, U>(mut self, f_titles: T) -> Self
    where
        T: IntoIterator<Item = U>,
        U: AsRef<str>,
    {
        self.action_values = self.action_values.titles(f_titles);
        self
    }

    pub fn pageids(mut self, f_pageids: &[u16]) -> Self {
        self.action_values = self.action_values.pageids(f_pageids);
        self
    }

    pub fn prop(mut self, f_prop: query::Prop) -> Self {
        self.action_values = self.action_values.prop(f_prop);
        self
    }

    pub fn list(mut self, f_list: query::List) -> Self {
        self.action_values = self.action_values.list(f_list);
        self
    }

    pub fn set_titles<T, U>(&mut self, titles: T)
    where
        T: IntoIterator<Item = U>,
        U: AsRef<str>,
    {
        self.validated = false;
        self.action_values.titles =
            Some(titles.into_iter().map(|s| s.as_ref().to_string()).collect());
    }

    pub fn set_pageids(&mut self, pageids: &[u16]) {
        self.validated = false;
        self.action_values.pageids = Some(pageids.to_vec());
    }

    pub fn set_prop(&mut self, prop: query::Prop) {
        self.validated = false;
        self.action_values.prop = Some(prop);
    }

    pub fn set_list(&mut self, list: query::List) {
        self.validated = false;
        self.action_values.list = Some(list);
    }

    pub fn push_title(&mut self, title: &str) {
        self.validated = false;
        if let Some(v) = &mut self.action_values.titles {
            v.push(title.to_string());
        } else {
            self.action_values.titles = Some(vec![title.to_string()]);
        }
    }

    pub fn push_pageid(&mut self, pgid: u16) {
        self.validated = false;
        if let Some(v) = &mut self.action_values.pageids {
            v.push(pgid);
        } else {
            self.action_values.pageids = Some(vec![pgid]);
        }
    }
}

// impl PARAMS<parse::Parse> {
//     fn new() -> Self {
//         Self {
//             action: Action::Parse,
//             action_values: parse::Parse::new(),
//             format: Format::Json,
//             errorformat: ErrorFormat::Raw,
//             validated: false,
//         }
//     }
// }

impl<T: ActionType> PARAMS<T> {
    pub fn validate(mut self) -> Result<Self> {
        if self.validated {
            debug_assert!(self.action_values.is_valid());
            Ok(self)
        } else if self.action_values.is_valid() {
            self.validated = true;
            Ok(self)
        } else {
            Err(Error::Args)
        }
    }

    pub fn format(mut self, f: Format) -> Self {
        self.format = f;
        self
    }

    pub fn set_format(&mut self, f: Format) {
        self.format = f;
    }

    pub fn err_format(mut self, f: ErrorFormat) -> Self {
        self.errorformat = f;
        self
    }

    pub fn set_err_format(&mut self, f: ErrorFormat) {
        self.errorformat = f;
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use query::*;
    use serde_json::{json, to_value};

    fn construct_unvalidated_but_valid_query_PARAMS() -> PARAMS<Query> {
        let q = Query::new()
            .pageids(&[100, 420, 8887, u16::MAX])
            .prop(Prop::Info)
            .validate()
            .unwrap();
        PARAMS::from_query(q)
    }

    #[test]
    fn test_PARAMS_inplace_construction1() {
        let control = construct_unvalidated_but_valid_query_PARAMS()
            .format(Format::JsonFm)
            .err_format(ErrorFormat::PlainText)
            .validate()
            .unwrap();

        let mut exp: PARAMS<Query> = PARAMS::new();
        exp.set_format(Format::JsonFm);
        exp.set_err_format(ErrorFormat::PlainText);
        exp.set_pageids(&[100, 420, 8887, u16::MAX]);
        exp.set_prop(Prop::Info);
        exp = exp.validate().unwrap();

        assert_eq!(control, exp)
    }

    #[test]
    fn test_PARAMS_chained_construction1() {
        let control = construct_unvalidated_but_valid_query_PARAMS()
            .validate()
            .unwrap();

        let exp: PARAMS<Query> = PARAMS::new()
            .pageids(&[100, 420, 8887, u16::MAX])
            .prop(Prop::Info)
            .format(Format::Json)
            .err_format(ErrorFormat::Raw)
            .validate()
            .unwrap();

        assert_eq!(control, exp)
    }

    #[test]
    fn test_PARAMS_validatation_success1() {
        let url = construct_unvalidated_but_valid_query_PARAMS();
        assert!(url.validate().is_ok())
    }

    #[test]
    fn test_PARAMS_validatation_fail1() {
        let url = construct_unvalidated_but_valid_query_PARAMS().pageids(&vec![]);
        assert!(url.validate().is_err())
    }
    #[test]
    fn test_PARAMS_serialize_success1() {
        let url = construct_unvalidated_but_valid_query_PARAMS();

        assert!(to_value(url).is_ok())
    }

    #[test]
    fn test_PARAMS_serialize_success2() {
        let val = to_value(construct_unvalidated_but_valid_query_PARAMS()).unwrap();

        let exp = json!({
            "action": "query",
            "pageids": [100, 420, 8887, 65535],
            "prop": "info",
            "format": "json",
            "errorformat": "raw"
        });

        assert_eq!(val, exp);
    }
}
