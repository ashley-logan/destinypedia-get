use serde::{Deserialize, Serialize};

mod query {
    use super::*;

    #[derive(Debug, Serialize)]
    pub struct Query {
        titles: Option<Vec<String>>,
        pageids: Option<Vec<u16>>,
        prop: Option<self::Prop>,
        list: Option<self::List>,
    }

    #[derive(Debug, Serialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum Prop {
        Info,
        PageImages,
        Images,
        ImageInfo,
        Categories,
    }
    #[derive(Debug, Serialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum List {
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
                list: None
            }
        }

    }

    impl Query {
        fn titles<T, U>(mut self, titles: T) -> Self 
        where 
            T: IntoIterator<Item = U>,
            U: AsRef<str>
        {
            self.titles = Some(titles.into_iter().map(|s| s.as_ref().to_string()).collect());
            self
        }

        fn pageids<T>(mut self, pageids: T) -> Self 
        where 
            T: IntoIterator<Item = u16>,
        {
            self.pageids = Some(pageids.into_iter().collect());
            self
        }

        fn prop(mut self, prop: self::Prop) -> Self {
            self.prop = Some(prop);
            self
        }

        fn list(mut self, list: self::List) -> Self {
            self.list = Some(list);
            self
        }

    }
}

mod parse {
    use super::*;
    #[derive(Debug, Serialize)]
    pub struct Parse {
        pageid: u16,
        page: String,
        prop: Prop,
    }

    #[derive(Debug, Serialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum Prop {
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
}

impl ActionType for parse::Parse {}

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
    #[test]
    fn test_query_serialize() {
        let q = query::Query {
            titles
        }
    }
}
