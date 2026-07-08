use std::{fmt::Display, str::FromStr};

// use super::schema::{categories, image_categories, images, subcategories};
use crate::Result;
use crate::bin_modules::DestinyFetchError;
use chrono::Utc;
use destinypedia::response::{Categories, CategoryInfo, ImageInfo, Images, QueryResult, items::*};
use sqlx::{
    FromRow, Row as Row_,
    sqlite::{Sqlite, SqliteRow},
};

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
    UNKNOWN,
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
            _ => Ok(Self::UNKNOWN),
        }
    }
}

impl From<String> for Ext {
    fn from(value: String) -> Self {
        match &value.to_lowercase()[..] {
            "png" => Self::PNG,
            "jpg" | "jpeg" => Self::JPG,
            "svg" => Self::SVG,
            "gif" => Self::SVG,
            "mp4" => Self::MP4,
            "mp3" => Self::MP3,
            "webp" => Self::WEBP,
            _ => Self::UNKNOWN,
        }
    }
}

impl Ext {
    pub fn as_ext(value: impl AsRef<str>) -> Self {
        let s = value.as_ref();
        if let Some(i) = s.rfind('.') {
            Ext::from_str(&s[i + 1..]).unwrap_or(Ext::UNKNOWN)
        } else {
            Ext::UNKNOWN
        }
    }
}

#[derive(Debug, Clone, FromRow, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct ImagesRow {
    pub id: i64,
    pub title: String,
    pub size: i64,
    pub width: i64,
    pub height: i64,
    pub url: String,
    pub timestamp: i64,
    pub extension: Ext,
}

#[derive(Debug, FromRow)]
pub struct CategoriesRow {
    pub id: i64,
    pub title: String,
    pub files: i64,
    pub subcats: i64,
}

#[derive(Debug, FromRow)]
pub struct ImageCategoryRow {
    pub image_id: i64,
    pub category_id: i64,
}

#[derive(Debug, FromRow)]
pub struct SubCategoryRow {
    pub category_id: i64,
    pub subcategory_id: i64,
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
                id: query.pageid.into(),
                title: {
                    match query.title.strip_prefix(r"Category:") {
                        Some(s) => s.into(),
                        None => query.title,
                    }
                },
                files: files.into(),
                subcats: subcats.into(),
            }),
            _ => Err(super::error::DatabaseError::IntoRowConvertError)?,
        }
    }
}

impl TryFrom<QueryResult> for ImagesRow {
    type Error = DestinyFetchError;
    fn try_from(query: QueryResult) -> Result<Self> {
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
                    id: query.pageid.into(),
                    extension: Ext::as_ext(&title_),
                    title: {
                        match title_.strip_prefix(r"File:") {
                            Some(s) => s.into(),
                            None => title_,
                        }
                    },
                    size: (size_ / 1024) as i64,
                    width: width.into(),
                    height: height.into(),
                    url,
                    timestamp: timestamp.timestamp(),
                }),

                _ => Err(super::error::DatabaseError::IntoRowConvertError)?,
            },
            _ => Err(super::error::DatabaseError::IntoRowConvertError)?,
        }
    }
}
