use crate::ActionType;
use serde::{Serialize, ser::SerializeStruct};

#[derive(Debug, PartialEq, Eq)]
pub struct Query {
    pub(crate) titles: Option<Vec<String>>,
    pub(crate) pageids: Option<Vec<u16>>,
    pub(crate) prop: Option<self::Prop>,
    pub(crate) list: Option<self::List>,
}

impl Serialize for Query {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Query", 4)?;

        if let Some(l) = &self.list {
            state.serialize_field("list", l)?;
            return state.end();
        }

        if let Some(ids) = &self.pageids {
            state.serialize_field("pageids", ids)?;
        } else if let Some(ts) = &self.titles {
            state.serialize_field("titles", ts)?;
        }

        state.serialize_field("prop", &self.prop)?;

        state.end()
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Prop {
    Info,
    PageImages,
    Images,
    ImageInfo,
    Categories,
}
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum List {
    AllImages,
    AllPages,
    AllCategories,
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

    pub fn pageids(mut self, pageids: &[u16]) -> Self {
        self.pageids = Some(pageids.to_vec());
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, to_value};

    fn construct_unvalidated_query() -> Query {
        Query::new()
            .titles(&["Example title", "Another example title"])
            .pageids(&[1333, 892, 999])
            .prop(Prop::Categories)
    }

    fn construct_valid_query() -> Query {
        construct_unvalidated_query().validate().unwrap()
    }

    #[test]
    fn test_validate_func_success() {
        assert!(construct_unvalidated_query().validate().is_ok())
    }

    #[test]
    fn test_validate_func_fail1() {
        // should fail because both prop and list are initialized
        let q = construct_unvalidated_query().list(List::AllLinks);
        assert!(q.validate().is_err())
    }

    #[test]
    fn test_validate_func_fail2() {
        // should fail because both titles and pageids are empty
        let q = construct_unvalidated_query()
            .titles::<Vec<_>, &str>(vec![])
            .pageids(&[]);

        assert!(q.validate().is_err())
    }

    #[test]
    fn test_query_serialize_success1() {
        let val = to_value(construct_valid_query());
        assert!(val.is_ok());
    }

    #[test]
    fn test_query_serialize_success2() {
        let val = to_value(construct_valid_query()).unwrap();

        let expected = json!({
            "pageids": [1333, 892, 999],
            "prop": "categories"
        });

        assert_eq!(val, expected);
    }
}
