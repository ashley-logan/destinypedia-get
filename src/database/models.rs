use crate::models::NAMESPACE;
use crate::models::deserialize::IndiscriminateQueryResult;
use crate::models::deserialize::{
    CategoriesProp, CategoryInfoProp, ImageInfoProp, ImagesProp, InfoProp, PageImagesProp,
};
use sqlx::FromRow;

pub struct ImagesRow<'a> {
    id: u32,
    title: &'a str,
    size: u128,
    width: usize,
    height: usize,
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

pub fn to_images_row(query: &IndiscriminateQueryResult) -> crate::Result<ImagesRow<'_>> {
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

pub fn to_categories_row(query: &IndiscriminateQueryResult) -> crate::Result<CategoriesRow<'_>> {
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

pub fn to_imagecategory_row(
    query: &IndiscriminateQueryResult,
) -> crate::Result<Vec<ImageCategoryRow<'_>>> {
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
