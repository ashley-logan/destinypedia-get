use crate::{Error, Result, query::Query};
use serde::Serialize;

pub trait ActionType: Serialize {
    fn new() -> Self;

    fn has_list(&self) -> bool {
        false
    }

    fn has_prop(&self) -> bool {
        false
    }

    fn has_generator(&self) -> bool {
        false
    }

    fn validate(self) -> Result<Self>
    where
        Self: Sized;

    fn is_valid(&self) -> bool;
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

    fn has_list(&self) -> bool {
        self.list.is_some()
    }

    fn has_prop(&self) -> bool {
        self.prop.is_some()
    }

    fn has_generator(&self) -> bool {
        todo!();
    }

    fn validate(self) -> Result<Self> {
        // make sure either titles or pageids is initialized with data
        match (&self.titles, &self.pageids) {
            (None, None) => return Err(Error::Args),
            (Some(v), None) => {
                if v.is_empty() {
                    return Err(Error::Args);
                }
            }
            (None, Some(v)) => {
                if v.is_empty() {
                    return Err(Error::Args);
                }
            }
            (Some(v1), Some(v2)) => {
                if v1.is_empty() && v2.is_empty() {
                    return Err(Error::Args);
                }
            }
        };

        // make sure exactly one of prop or list is initialized with data
        match (&self.prop, &self.list) {
            (Some(_), Some(_)) => Err(Error::Args), // error since neither prop nor list take priority
            (None, None) => Err(Error::Args),
            _ => Ok(self),
        }
    }

    fn is_valid(&self) -> bool {
        // make sure either titles or pageids is initialized with data
        match (&self.titles, &self.pageids) {
            (None, None) => return false,
            (Some(v), None) => {
                if v.is_empty() {
                    return false;
                }
            }
            (None, Some(v)) => {
                if v.is_empty() {
                    return false;
                }
            }
            (Some(v1), Some(v2)) => {
                if v1.is_empty() && v2.is_empty() {
                    return false;
                }
            }
        };

        // make sure exactly one of prop or list is initialized with data
        match (&self.prop, &self.list) {
            (Some(_), Some(_)) => false, // error since neither prop nor list take priority
            (None, None) => false,
            _ => true,
        }
    }
}
