use crate::models::NAMESPACE;
use crate::models::deserialize::QueryResult;
use crate::models::deserialize::items::*;
use crate::models::deserialize::{
    CategoriesProp, CategoryInfoProp, ImageInfoProp, ImagesProp, InfoProp, PageImagesProp,
};

pub enum Row {
    Images(ImagesRow),
    Categories(CategoriesRow),
    ImageCategory(ImageCategoryRow),
    SubCategory(SubCategoryRow),
}

pub struct ImagesRow {
    pub id: u32,
    pub title: String,
    pub size: u128,
    pub width: u32,
    pub height: u32,
    pub url: String,
    pub timestamp: String,
    // category_titles: Vec<String>
}
pub struct CategoriesRow {
    pub id: u32,
    pub title: String,
    pub files: u32,
    pub subcats: u32,
}

pub struct ImageCategoryRow {
    pub image_id: u32,
    pub category_id: u32,
}

pub struct SubCategoryRow {
    pub id: u32,
    pub subcategory_id: u32,
}

impl From<(u32, u32)> for SubCategoryRow {
    fn from(value: (u32, u32)) -> Self {
        Self {
            id: value.0,
            subcategory_id: value.1,
        }
    }
}

pub fn into_images_row(query: QueryResult) -> crate::Result<ImagesRow> {
    debug_assert!(!matches!(query.ns, NAMESPACE::FILE)); // this check should happen in the caller
    match query.imageinfo.map(|ii| ii.0.into_iter().next()) {
        Some(Some(item)) => match item {
            ImageInfoItem {
                canonicaltitle: Some(title),
                size: Some(size),
                width: Some(width),
                height: Some(height),
                url: Some(url),
                timestamp: Some(timestamp),
            } => Ok(ImagesRow {
                id: query.pageid,
                title,
                size,
                width,
                height,
                url,
                timestamp,
            }),

            _ => Err(crate::Error::TryFromResponseIntoRow),
        },
        _ => Err(crate::Error::TryFromResponseIntoRow),
    }
}

pub fn into_categories_row(query: QueryResult) -> crate::Result<CategoriesRow> {
    debug_assert!(!matches!(query.ns, NAMESPACE::CATEGORY)); // this check should happen in the caller
    match query.categoryinfo {
        Some(CategoryInfoProp(CatgeoryInfoItem {
            files: Some(files_),
            subcats: Some(subcats_),
            ..
        })) => Ok(CategoriesRow {
            id: query.pageid,
            title: query.title,
            files: files_,
            subcats: subcats_,
        }),
        _ => Err(crate::Error::TryFromResponseIntoRow),
    }
}

/// Super cheap to clone, so QueryResult is taken by reference.
/// as a rule of thumb, methods converting a QueryResult into a purely relational table row should be called
/// before methods converting a QueryResult into a regular, data-heavy table row.
/// This is because it's cheaper to crate relational rows and therefore they are created via cloning
/// rather than transferring ownership

mod ref_rows {
    use super::*;
    pub struct ImagesRow<'a> {
        id: u32,
        title: &'a str,
        size: u128,
        width: u32,
        height: u32,
        url: &'a str,
        timestamp: &'a str,
        // category_titles: Vec<String>
    }

    pub struct CategoriesRow<'a> {
        id: u32,
        title: &'a str,
        parent_category_titles: Vec<&'a str>,
    }

    pub struct ImageCategoryRow<'a> {
        image_id: u32,
        category_title: &'a str,
    }

    pub struct SubCategoryRow {
        category_id: u32,
        subcategory_id: u32,
    }

    pub fn to_images_row(query: &QueryResult) -> crate::Result<ImagesRow<'_>> {
        if !matches!(query.ns, NAMESPACE::FILE) {
            return Err(crate::Error::TryFromResponseIntoRow);
        }
        if let Some(prop) = &query.imageinfo {
            if let Some(ii) = prop.0.iter().next() {
                let size = ii.size.ok_or(crate::Error::TryFromResponseIntoRow)?;
                let width = ii.width.ok_or(crate::Error::TryFromResponseIntoRow)?;
                let height = ii.height.ok_or(crate::Error::TryFromResponseIntoRow)?;
                let url: &str = ii
                    .url
                    .as_ref()
                    .ok_or(crate::Error::TryFromResponseIntoRow)?;
                let timestamp: &str = ii
                    .timestamp
                    .as_ref()
                    .ok_or(crate::Error::TryFromResponseIntoRow)?;

                let id = query.pageid;
                let title: &str = &query.title;

                return Ok(ImagesRow {
                    id,
                    title,
                    size,
                    width,
                    height,
                    url,
                    timestamp,
                });
            }
        }
        Err(crate::Error::TryFromResponseIntoRow)
    }

    pub fn to_categories_row(query: &QueryResult) -> crate::Result<CategoriesRow<'_>> {
        if !matches!(query.ns, NAMESPACE::CATEGORY) {
            return Err(crate::Error::TryFromResponseIntoRow);
        }
        if let Some(cprop) = &query.categories {
            let parent_category_titles: Vec<&str> =
                cprop.0.iter().map(|ci| ci.title.as_str()).collect();
            let id = query.pageid;
            let title: &str = query.title.as_str();

            return Ok(CategoriesRow {
                id,
                title,
                parent_category_titles,
            });
        }

        Err(crate::Error::TryFromResponseIntoRow)
    }

    pub fn to_imagecategory_row(query: &QueryResult) -> crate::Result<Vec<ImageCategoryRow<'_>>> {
        if !matches!(query.ns, NAMESPACE::FILE) {
            return Err(crate::Error::TryFromResponseIntoRow);
        }
        if let Some(cprop) = &query.categories {
            let rows: Vec<ImageCategoryRow> = cprop
                .0
                .iter()
                .map(|ci| ImageCategoryRow {
                    image_id: query.pageid,
                    category_title: ci.title.as_str(),
                })
                .collect();
            return Ok(rows);
        }
        Err(crate::Error::TryFromResponseIntoRow)
    }
}
