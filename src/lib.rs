mod error;
use error::Result;
use serde::{Deserialize, Serialize};

mod query {
    use super::*;

    #[derive(Debug, Serialize)]
    pub struct Query {
        #[serde(skip_serializing_if = "Option::is_none")]
        titles: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pageids: Option<Vec<u16>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        prop: Option<self::Prop>,
        #[serde(skip_serializing_if = "Option::is_none")]
        list: Option<self::List>,
    }

    #[derive(Debug, Serialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub(super) enum Prop {
        Info,
        PageImages,
        Images,
        ImageInfo,
        Categories,
    }
    #[derive(Debug, Serialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub(super) enum List {
        AllImages,
        AllLinks,
        AllPages,
    }

    impl ActionType for Query {
        fn new() -> Self {
            Self {
                titles: None,
                pageids: None,
                prop: None,
                list: None,
            }
        }

        fn validate(&self) -> super::Result<()> {
            // make sure either titles or pageids is initialized with data
            match (&self.titles, &self.pageids) {
                (None, None) => return Err(error::Error::Args),
                (Some(v), None) => {
                    if v.is_empty() {
                        return Err(error::Error::Args);
                    }
                }
                (None, Some(v)) => {
                    if v.is_empty() {
                        return Err(error::Error::Args);
                    }
                }
                _ => (),
            };

            // make sure exactly one of prop or list is initialized with data
            match (&self.prop, &self.list) {
                (Some(_), Some(_)) => Err(error::Error::Args), // error since neither prop nor list take priority
                (None, None) => Err(error::Error::Args),
                _ => Ok(()),
            }
        }
    }

    impl Query {
        pub fn titles<T, U>(mut self, titles: T) -> Self
        where
            T: IntoIterator<Item = U>,
            U: AsRef<str>,
        {
            self.titles = Some(titles.into_iter().map(|s| s.as_ref().to_string()).collect());
            self
        }

        pub fn pageids<T>(mut self, pageids: T) -> Self
        where
            T: IntoIterator<Item = u16>,
        {
            self.pageids = Some(pageids.into_iter().collect());
            self
        }

        pub fn prop(mut self, prop: self::Prop) -> Self {
            self.prop = Some(prop);
            self
        }

        pub fn list(mut self, list: self::List) -> Self {
            self.list = Some(list);
            self
        }
    }
}

mod parse {
    use super::*;
    #[derive(Debug, Serialize)]
    pub(super) struct Parse {
        pageid: u16,
        page: String,
        prop: Prop,
    }

    #[derive(Debug, Serialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub(super) enum Prop {
        Images,
        TocData,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum Action {
    Query,
    Parse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum ErrorFormat {
    PlainText,
    WikiText,
    HTML,
    Raw,
    None,
    BC,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum Format {
    Json,
    JsonFm,
    None,
    Php,
    PhpFm,
    RawFm,
    Xml,
    XmlFm,
}

pub trait ActionType: Serialize {
    fn new() -> Self;

    fn validate(&self) -> Result<()>;
}

#[derive(Debug, Serialize)]
pub struct URL<T: ActionType> {
    action: Action,
    #[serde(flatten)]
    extra: T,
    format: Format,
    errorformat: ErrorFormat,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_value;
    #[test]
    fn test_query_serialize_success1() {
        let q = query::Query::new()
            .titles(&["Example title", "Another example title"])
            .prop(query::Prop::Categories);
        let val = to_value(q);

        assert!(val.is_ok());
    }

    #[test]
    fn test_query_serialize_fail1() {
        let q = query::Query::new()
            .titles(&["Example title", "Another example title"])
            .prop(query::Prop::Categories)
            .list(query::List::AllLinks);

        todo!("Force validate function to run to qualify for serialization");
        let val = to_value(q);
    }
}
