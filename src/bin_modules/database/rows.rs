use super::schema::{categories, image_categories, images, subcategories};
use destinypedia::NAMESPACE;
use destinypedia::response::{Categories, CategoryInfo, ImageInfo, Images, QueryResult, items::*};
use diesel::prelude::Insertable;

#[derive(Debug)]
pub enum Row<'a> {
    Images(ImagesRow<'a>),
    Categories(CategoriesRow<'a>),
    ImageCategory(ImageCategoryRow<'a>),
    SubCategory(SubCategoryRow<'a>),
}

#[derive(Debug, Insertable)]
#[diesel(table_name = images)]
pub struct ImagesRow<'a> {
    pub id: &'a u32,
    pub title: &'a str,
    pub size: &'a u32,
    pub width: &'a u32,
    pub height: &'a u32,
    pub url: &'a str,
    pub timestamp: &'a str,
    // category_titles: Vec<String>
}
#[derive(Debug, Insertable)]
#[diesel(table_name = categories)]
pub struct CategoriesRow<'a> {
    pub id: &'a u32,
    pub title: &'a str,
    pub files: &'a u32,
    pub subcats: &'a u32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = image_categories)]
pub struct ImageCategoryRow<'a> {
    pub image_id: &'a u32,
    pub category_id: &'a u32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = subcategories)]
pub struct SubCategoryRow<'a> {
    pub category_id: &'a u32,
    pub subcategory_id: &'a u32,
}

impl From<(u32, u32)> for SubCategoryRow {
    fn from(value: (u32, u32)) -> Self {
        Self {
            id: value.0,
            subcategory_id: value.1,
        }
    }
}

impl TryFrom<QueryResult> for CategoriesRow {
    type Error = crate::bin_modules::DestinyFetchError;
    fn try_from(query: QueryResult) -> Result<Self, Self::Error> {
        match query.categoryinfo {
            Some(CategoryInfo(CategoryInfoItem {
                files: Some(files_),
                subcats: Some(subcats_),
                ..
            })) => Ok(CategoriesRow {
                id: query.pageid,
                title: {
                    match query.title.strip_prefix(r"Category:") {
                        Some(s) => s.into(),
                        None => query.title,
                    }
                },
                files: files_,
                subcats: subcats_,
            }),
            _ => Err(super::error::DatabaseError::IntoRowConvertError)?,
        }
    }
}

impl TryFrom<QueryResult> for ImagesRow {
    type Error = crate::bin_modules::DestinyFetchError;
    fn try_from(query: QueryResult) -> Result<Self, Self::Error> {
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
