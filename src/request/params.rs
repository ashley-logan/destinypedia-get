use super::query_objs::Action;
use super::{Generator, Prop, Query, error::RequestResult};
use derive_more::Display;
use serde::Serialize;
use serde_json::{Map, Value};
use serde_with::{DisplayFromStr, serde_as};
use std::marker::PhantomData;

pub struct ParamsBuilder<T: Action> {
    params: T,
    format: Option<Format>,
    extra: Option<Map<String, Value>>,
}

#[serde_as]
#[derive(Debug, Serialize, Default)]
pub struct PARAMS<T: Action> {
    #[serde(flatten)]
    params: serde_json::Value,
    #[serde_as(as = "DisplayFromStr")]
    format: Format,
    #[serde(skip)]
    action_marker: PhantomData<T>,
}

#[derive(Debug, Display, Default)]
#[display(rename_all = "lowercase")]
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

#[derive(Debug, Display, Default)]
#[display(rename_all = "lowercase")]
pub enum ErrorFormat {
    PlainText,
    WikiText,
    HTML,
    #[default]
    Raw,
    None,
    BC,
}

impl<T: Action> PARAMS<T> {
    pub fn build() -> ParamsBuilder<T> {
        ParamsBuilder::new()
    }
}

impl PARAMS<Query> {
    pub fn update_continue(
        &mut self,
        cont_key: impl Into<String>,
        cont_value: impl Into<String>,
    ) -> Option<String> {
        if let Some(obj) = self.params.as_object_mut() {
            obj.insert(cont_key.into(), cont_value.into().into())
                .map(|v| v.to_string())
        } else {
            None
        }
    }

    pub fn remove_continue(&mut self, cont_key: &str) -> Option<String> {
        if let Some(obj) = self.params.as_object_mut() {
            obj.remove_entry(cont_key)
                .map(|(_, v)| return v.to_string())
        } else {
            None
        }
    }
}

impl<T: Action> ParamsBuilder<T> {
    pub fn new() -> Self {
        Self {
            params: T::default(),
            format: None,
            extra: None,
        }
    }

    pub fn build(self) -> RequestResult<PARAMS<T>> {
        let mut val = serde_json::to_value(self.params)?;
        if let Some(obj) = val.as_object_mut() {
            if obj.contains_key("pageids") && obj.contains_key("titles") {
                obj.remove_entry("titles");
            }
            if let Some(extra) = self.extra {
                obj.extend(extra.into_iter());
            }
        }
        Ok(PARAMS {
            params: val,
            format: self.format.unwrap_or_default(),
            action_marker: PhantomData,
        })
    }

    pub fn with_format(mut self, format_: Format) -> Self {
        self.format = Some(format_);
        self
    }

    pub fn set_format(&mut self, format_: Format) {
        self.format = Some(format_);
    }

    pub fn with_extra(mut self, key: impl Into<String>, val: impl Into<Value>) -> Self {
        self.extra
            .get_or_insert_with(Map::new)
            .insert(key.into(), val.into());
        self
    }

    pub fn with_extras(
        mut self,
        extras: impl IntoIterator<Item = (impl Into<String>, impl Into<Value>)>,
    ) -> Self {
        let map = extras.into_iter().map(|(k, v)| (k.into(), v.into()));
        self.extra.get_or_insert_with(Map::new).extend(map);
        self
    }

    pub fn extend_extras(
        &mut self,
        extras: impl IntoIterator<Item = (impl Into<String>, impl Into<Value>)>,
    ) {
        self.extra
            .get_or_insert_with(Map::new)
            .extend(extras.into_iter().map(|(k, v)| (k.into(), v.into())));
    }

    pub fn append_extra(&mut self, key: String, vals: impl IntoIterator<Item = impl Into<String>>) {
        let val: Vec<String> = vals.into_iter().map(|x| x.into()).collect();
        let s: String = val.join(r"|");
        self.extra
            .get_or_insert_with(Map::new)
            .insert(key, s.into());
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
    pub fn with_pageids(mut self, pageids_: impl IntoIterator<Item = impl Into<i32>>) -> Self {
        self.params.pageids = Some(pageids_.into_iter().map(Into::into).collect());

        self
    }

    pub fn with_continue(mut self, ckey: impl Into<String>, cval: impl Into<String>) -> Self {
        self.params.cont = Some((ckey.into(), cval.into()));

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

    pub fn with_indexpageids(mut self, ind: bool) -> Self {
        self.params.indexpageids = ind;
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
    pub fn append_pageids(&mut self, pageids_: impl IntoIterator<Item = impl Into<i32>>) {
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

    pub fn set_indexpageids(&mut self, ind: bool) {
        self.params.indexpageids = ind;
    }

    pub fn set_continue(&mut self, ckey: impl Into<String>, cval: impl Into<String>) {
        self.params.cont = Some((ckey.into(), cval.into()));
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
        pageids_: Option<impl IntoIterator<Item = impl Into<i32>>>,
    ) -> Option<Vec<i32>> {
        let prev: Option<Vec<i32>> = self.params.pageids.take();

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
        let prev: Option<(String, String)> = self.params.cont.take();

        if let Some((k, v)) = continue_ {
            self.params.cont = Some((k.into(), v.into()));
        }

        prev
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{Token, assert_ser_tokens};

    #[test]
    fn test_chain_builder() {
        let params: PARAMS<Query> = PARAMS::build()
            .with_pageids([300_i32, 400_i32, 500_i32])
            .with_props([Prop::PageImages, Prop::Info])
            .with_format(Format::XmlFm)
            .build()
            .unwrap();

        assert_ser_tokens(
            &params,
            &[
                Token::Map { len: None },
                Token::Str("action"),
                Token::Str("query"),
                Token::Str("pageids"),
                Token::Str("300|400|500"),
                Token::Str("prop"),
                Token::Str("pageimages|info"),
                Token::Str("format"),
                Token::Str("xmlfm"),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn test_inplace_builder() {
        let mut builder: ParamsBuilder<Query> = ParamsBuilder::new();
        builder.append_titles(["oneTitle", "twoTitle"]);
        builder.append_props([Prop::Images, Prop::ImageInfo]);
        builder.replace_or_remove_titles(Some(["redTitle", "blueTitle"]));
        builder.set_continue("continue", "someValue");
        builder.set_continue("continue", "actualValue");
        let params: PARAMS<Query> = builder.build().unwrap();

        assert_ser_tokens(
            &params.params,
            &[
                Token::Map { len: Some(4) },
                Token::Str("action"),
                Token::Str("query"),
                Token::Str("continue"),
                Token::Str("actualValue"),
                Token::Str("prop"),
                Token::Str("images|imageinfo"),
                Token::Str("titles"),
                Token::Str("redTitle|blueTitle"),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn test_mixed_builder() {
        let mut builder: ParamsBuilder<Query> = PARAMS::build()
            .with_props([Prop::ImageInfo])
            .with_continue("remove", "later");

        builder.set_format(Format::Php);

        let g = Generator::allimages_with(Some("prefix".into()), None);

        builder.set_generator(g);

        let mut params: PARAMS<Query> = builder.build().unwrap();

        assert!(params.remove_continue("remove").is_some());

        assert_ser_tokens(
            &params,
            &[
                Token::Map { len: None },
                Token::Str("action"),
                Token::Str("query"),
                Token::Str("gailimit"),
                Token::Str("20"),
                Token::Str("gaiprefix"),
                Token::Str("prefix"),
                Token::Str("generator"),
                Token::Str("allimages"),
                Token::Str("prop"),
                Token::Str("imageinfo"),
                Token::Str("format"),
                Token::Str("php"),
                Token::MapEnd,
            ],
        );
    }
}
