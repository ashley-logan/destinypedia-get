use super::schema::{categories, image_categories, images, subcategories};
use crate::Result;
use crate::bin_modules::DestinyFetchError;
use destinypedia::response::{Categories, CategoryInfo, ImageInfo, Images, QueryResult, items::*};
use diesel::prelude::Insertable;

#[derive(Debug)]
pub enum Row {
    Images(ImagesRow),
    Categories(CategoriesRow),
    ImageCategory(ImageCategoryRow),
    SubCategory(SubCategoryRow),
}

#[derive(Debug, Insertable)]
#[diesel(table_name = images)]
pub struct ImagesRow {
    pub id: i32,
    pub title: String,
    pub size: i32,
    pub width: i32,
    pub height: i32,
    pub url: String,
    pub timestamp: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = categories)]
pub struct CategoriesRow {
    pub id: i32,
    pub title: String,
    pub files: i32,
    pub subcats: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = image_categories)]
pub struct ImageCategoryRow {
    pub image_id: i32,
    pub category_id: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = subcategories)]
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
                    timestamp,
                }),

                _ => Err(super::error::DatabaseError::IntoRowConvertError)?,
            },
            _ => Err(super::error::DatabaseError::IntoRowConvertError)?,
        }
    }
}
