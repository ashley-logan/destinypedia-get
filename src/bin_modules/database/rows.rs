use std::{fmt::Display, str::FromStr};

// use super::schema::{categories, image_categories, images, subcategories};
use crate::Result;
use crate::bin_modules::DestinyFetchError;
use chrono::Utc;
use destinypedia::response::{Categories, CategoryInfo, ImageInfo, Images, QueryResult, items::*};
use sqlx::{FromRow, sqlite::Sqlite};

#[derive(Debug)]
pub enum Row {
    Images(ImagesRow),
    Categories(CategoriesRow),
    ImageCategory(ImageCategoryRow),
    SubCategory(SubCategoryRow),
}
#[serde_with::serde_as]
#[derive(
    Debug,
    Clone,
    clap::ValueEnum,
    sqlx::Type,
    derive_more::Display,
    PartialEq,
    Eq,
    serde_with::SerializeDisplay,
    serde_with::DeserializeFromStr,
)]
#[display(rename_all = "lowercase")]
pub enum Ext {
    PNG,
    JPG,
    SVG,
    GIF,
    MP4,
    MP3,
    WEBP,
}
impl std::str::FromStr for Ext {
    type Err = DestinyFetchError;
    fn from_str(s: &str) -> Result<Self> {
        match &s.to_lowercase()[..] {
            "png" => Ok(Self::PNG),
            "jpg" | "jpeg" => Ok(Self::JPG),
            "svg" => Ok(Self::SVG),
            "gif" => Ok(Self::SVG),
            "mp4" => Ok(Self::MP4),
            "mp3" => Ok(Self::MP3),
            "webp" => Ok(Self::WEBP),
            _ => Err(DestinyFetchError::ExtFromStrErr),
        }
    }
}
impl Ext {
    pub fn as_ext(value: impl AsRef<str>) -> Option<Self> {
        let s = value.as_ref();
        if let Some(i) = s.rfind('.') {
            Ext::from_str(&s[i + 1..]).ok()
        } else {
            None
        }
    }
}

#[derive(Debug, FromRow, serde::Deserialize, serde::Serialize)]
pub struct ImagesRow {
    pub id: i32,
    pub title: String,
    pub size: i32,
    pub width: i32,
    pub height: i32,
    pub url: String,
    pub timestamp_: chrono::NaiveDateTime,
    pub ext_: Option<Ext>,
}

#[derive(Debug, FromRow)]
pub struct CategoriesRow {
    pub id: i32,
    pub title: String,
    pub files: i32,
    pub subcats: i32,
}

#[derive(Debug, FromRow)]
pub struct ImageCategoryRow {
    pub image_id: i32,
    pub category_id: i32,
}

#[derive(Debug, FromRow)]
pub struct SubCategoryRow {
    pub category_id: i32,
    pub subcategory_id: i32,
}

impl TryFrom<QueryResult> for CategoriesRow {
    type Error = DestinyFetchError;
    fn try_from(query: QueryResult) -> Result<Self> {
        match query.categoryinfo {
            Some(CategoryInfo(CategoryInfoItem {
                files: Some(files),
                subcats: Some(subcats),
                ..
            })) => Ok(CategoriesRow {
                id: query.pageid,
                title: {
                    match query.title.strip_prefix(r"Category:") {
                        Some(s) => s.into(),
                        None => query.title,
                    }
                },
                files,
                subcats,
            }),
            _ => Err(super::error::DatabaseError::IntoRowConvertError)?,
        }
    }
}

impl TryFrom<QueryResult> for ImagesRow {
    type Error = DestinyFetchError;
    fn try_from(query: QueryResult) -> Result<Self> {
        use chrono::prelude::*;
        match query.imageinfo.map(|ii| ii.into_items().into_iter().next()) {
            Some(Some(item)) => match item {
                ImageInfoItem {
                    canonicaltitle: Some(title_),
                    size: Some(size_),
                    width: Some(width),
                    height: Some(height),
                    url: Some(url),
                    timestamp: Some(timestamp),
                } => Ok(ImagesRow {
                    id: query.pageid,
                    ext_: Ext::as_ext(&title_),
                    title: {
                        match title_.strip_prefix(r"File:") {
                            Some(s) => s.into(),
                            None => title_,
                        }
                    },
                    size: size_ / 1024,
                    width,
                    height,
                    url,
                    timestamp_: timestamp.naive_utc(),
                }),

                _ => Err(super::error::DatabaseError::IntoRowConvertError)?,
            },
            _ => Err(super::error::DatabaseError::IntoRowConvertError)?,
        }
    }
}
