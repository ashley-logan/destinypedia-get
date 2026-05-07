use super::query::{Generator, Prop, Query};
use super::ser_types::Action;
use crate::{Error, Result};
use derive_more::Display;
use serde::Serialize;
use serde_with_macros::skip_serializing_none;

#[derive(Debug, Serialize, derive_more::PartialEq, derive_more::Eq, Display, Default)]
#[serde(rename_all = "lowercase")]
pub enum ErrorFormat {
    PlainText,
    WikiText,
    HTML,
    #[default]
    Raw,
    None,
    BC,
}

#[derive(Debug, Serialize, derive_more::PartialEq, derive_more::Eq, Display, Default)]
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

pub struct ParamsBuilder<T: Action> {
    params: T,
    continue_: Option<(String, String)>,
    format: Option<Format>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, derive_more::PartialEq, derive_more::Eq, Default)]
pub struct PARAMS {
    #[serde(flatten)]
    params: serde_json::Value,
    #[serde(flatten)]
    continue_: Option<(String, String)>,
    format: Format,
}

impl PARAMS {
    pub fn build<T: Action>() -> ParamsBuilder<T> {
        ParamsBuilder::new()
    }

    pub fn set_continue(&mut self, cont_key: &str, cont_value: &str) {
        if let Some((ck, cv)) = &mut self.continue_ {
            ck.clear();
            cv.clear();
            ck.push_str(cont_key);
            cv.push_str(cont_value);
        } else {
            self.continue_ = Some((cont_key.to_string(), cont_value.to_string()));
        }
    }
}

impl<T: Action> ParamsBuilder<T> {
    pub fn new() -> Self {
        Self {
            params: T::default(),
            continue_: None,
            format: None,
        }
    }

    pub fn build(self) -> Result<PARAMS> {
        let mut val = serde_json::to_value(self.params)?;
        if let Some(obj) = val.as_object_mut() {
            if obj.contains_key("pageids") && obj.contains_key("titles") {
                obj.remove_entry("titles");
            }
        }
        Ok(PARAMS {
            params: val,
            continue_: self.continue_,
            format: self.format.unwrap_or_default(),
        })
    }

    pub fn with_continue(mut self, ckey: impl Into<String>, cval: impl Into<String>) -> Self {
        self.continue_ = Some((ckey.into(), cval.into()));

        self
    }

    pub fn with_format(mut self, format_: Format) -> Self {
        self.format = Some(format_);
        self
    }

    pub fn set_continue_value(&mut self, cval: impl Into<String>) -> Result<()> {
        if let Some(tup) = &mut self.continue_ {
            tup.1 = cval.into();
            Ok(())
        } else {
            Err(Error::Params)
        }
    }

    pub fn set_continue_key(&mut self, ckey: impl Into<String>) -> Result<()> {
        if let Some(tup) = &mut self.continue_ {
            tup.0 = ckey.into();
            Ok(())
        } else {
            Err(Error::Params)
        }
    }

    pub fn set_continue(&mut self, ckey: impl Into<String>, cval: impl Into<String>) {
        self.continue_ = Some((ckey.into(), cval.into()));
    }

    pub fn set_format(&mut self, format_: Format) {
        self.format = Some(format_);
    }
}

impl ParamsBuilder<Query> {
    /// Chaining method for setting titles
    /// Overwrites any titles values with the provided collection
    /// Preferred for a builder-style interface
    pub fn with_titles(mut self, titles_: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.params.titles = Some(titles_.into_iter().map(Into::into).collect());

        self
    }

    /// Chaining method for setting pageids
    /// Overwrites any pageids values with the provided collection
    /// Preferred for a builder-style interface
    pub fn with_pageids(mut self, pageids_: impl IntoIterator<Item = impl Into<u32>>) -> Self {
        self.params.pageids = Some(pageids_.into_iter().map(Into::into).collect());

        self
    }

    /// Chaining method for setting props
    /// Overwrites any prop values with the provided collection
    /// Preferred for a builder-style interface
    pub fn with_props(mut self, props_: impl IntoIterator<Item = impl Into<Prop>>) -> Self {
        self.params.prop = Some(props_.into_iter().map(Into::into).collect());

        self
    }
    /// Chaining method for setting a generator
    /// Overwrites any current generator with the provided generator
    /// Preferred for a builder-style interface
    pub fn with_generator(mut self, generator_: Generator) -> Self {
        self.params.generator = Some(generator_);

        self
    }

    /// Inplace method for adding titles
    /// Appends the provided collection of titles to self
    /// Can be used even when self.params.titles is None
    pub fn append_titles(&mut self, titles_: impl IntoIterator<Item = impl Into<String>>) {
        self.params
            .titles
            .get_or_insert_with(Vec::new)
            .extend(titles_.into_iter().map(Into::into));
    }
    /// Inplace method for adding pageids
    /// Appends the provided collection of pageids to self
    /// Can be used even when self.params.pageids is None
    pub fn append_pageids(&mut self, pageids_: impl IntoIterator<Item = impl Into<u32>>) {
        self.params
            .pageids
            .get_or_insert_with(Vec::new)
            .extend(pageids_.into_iter().map(Into::into));
    }

    pub fn append_props(&mut self, props_: impl IntoIterator<Item = Prop>) {
        self.params
            .prop
            .get_or_insert_with(Vec::new)
            .extend(props_.into_iter());
    }

    pub fn set_generator(&mut self, generator_: Generator) {
        self.params.generator = Some(generator_);
    }

    /// Replaces or Removes the current titles Vec and returns it
    /// If titles_ is Some, params.titles is replaced with the provided collection
    /// If titles_ is None, params.titles is set to None
    /// When the caller doesn't care about the previous value, prefer replacing with the 'append_titles' method
    pub fn replace_or_remove_titles(
        &mut self,
        titles_: Option<impl IntoIterator<Item = impl Into<String>>>,
    ) -> Option<Vec<String>> {
        let prev: Option<Vec<String>> = self.params.titles.take();

        if let Some(it) = titles_ {
            self.params.titles = Some(it.into_iter().map(Into::into).collect());
        }

        prev
    }

    /// Replaces or Removes the current pageids Vec and returns it
    /// If pageids_ is Some, params.pageids is replaced with the provided collection
    /// If pageids_ is None, params.pageids is set to None
    /// When the caller doesn't care about the previous value, prefer replacing with the 'append_pageids' method
    pub fn replace_or_remove_pageids(
        &mut self,
        pageids_: Option<impl IntoIterator<Item = impl Into<u32>>>,
    ) -> Option<Vec<u32>> {
        let prev: Option<Vec<u32>> = self.params.pageids.take();

        if let Some(it) = pageids_ {
            self.params.pageids = Some(it.into_iter().map(Into::into).collect());
        }

        prev
    }

    /// Replaces or Removes the current cont (key, value) tuple and returns it
    /// If continue_ is Some, cont is replaces with the provided tuple
    /// If continue_ is None, cont is set to None
    /// When the caller doesn't care about the previous value, prefer replacing with the 'set_continue' method
    pub fn replace_or_remove_continue(
        &mut self,
        continue_: Option<(impl Into<String>, impl Into<String>)>,
    ) -> Option<(String, String)> {
        let prev: Option<(String, String)> = self.continue_.take();

        if let Some((k, v)) = continue_ {
            self.continue_ = Some((k.into(), v.into()));
        }

        prev
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::super::ser_types::Limit;
    use super::*;
    use serde_json::{json, to_value};
    use serde_test::{Token, assert_ser_tokens};

    #[test]
    fn test_chain_builder() {
        let params: PARAMS = PARAMS::build::<Query>()
            .with_pageids([300_u32, 400_u32, 500_u32])
            .with_props([Prop::PageImages, Prop::Info])
            .with_format(Format::XmlFm)
            .build()
            .unwrap();

        assert_ser_tokens(&params.params, &[todo!()]);
    }

    #[test]
    fn test_inplace_builder() {
        let mut builder: ParamsBuilder<Query> = ParamsBuilder::new();
        builder.append_titles(["oneTitle", "twoTitle"]);
        builder.append_props([Prop::Images, Prop::ImageInfo]);
        builder.replace_or_remove_titles(Some(["redTitle", "blueTitle"]));

        let params: PARAMS = builder.build().unwrap();

        assert_ser_tokens(&params.params, &[todo!()]);
    }
}
