use destinypedia::NAMESPACE;
use destinypedia::response::{Categories, CategoryInfo, ImageInfo, Images, QueryResult, items::*};

pub enum Row {
    Images(ImagesRow),
    Categories(CategoriesRow),
    ImageCategory(ImageCategoryRow),
    SubCategory(SubCategoryRow),
}

pub struct ImagesRow {
    pub id: u32,
    pub title: String,
    pub size: f32,
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

pub fn into_images_row(query: QueryResult) -> super::error::DatabaseResult<ImagesRow> {
    debug_assert!(matches!(query.ns, NAMESPACE::FILE)); // this check should happen in the caller
    match query.imageinfo.map(|ii| ii.into_items().into_iter().next()) {
        Some(Some(item)) => match item {
            ImageInfoItem {
                canonicaltitle: Some(title),
                size: Some(size_),
                width: Some(width),
                height: Some(height),
                url: Some(url),
                timestamp: Some(timestamp),
            } => Ok(ImagesRow {
                id: query.pageid,
                title,
                size: (size_ as f32) / 1024_f32,
                width,
                height,
                url,
                timestamp,
            }),

            _ => Err(super::error::DatabaseError::IntoRowConvertError),
        },
        _ => Err(super::error::DatabaseError::IntoRowConvertError),
    }
}

pub fn into_categories_row(query: QueryResult) -> super::error::DatabaseResult<CategoriesRow> {
    debug_assert!(matches!(query.ns, NAMESPACE::CATEGORY)); // this check should happen in the caller
    match query.categoryinfo {
        Some(CategoryInfo(CategoryInfoItem {
            files: Some(files_),
            subcats: Some(subcats_),
            ..
        })) => Ok(CategoriesRow {
            id: query.pageid,
            title: query.title,
            files: files_,
            subcats: subcats_,
        }),
        _ => Err(super::error::DatabaseError::IntoRowConvertError),
    }
}
