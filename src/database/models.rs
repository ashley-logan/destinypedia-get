use sqlx::FromRow;
use crate::models::deserialize::IndiscriminateQueryResult;
use crate::models::deserialize::{CategoriesProp, CategoryInfoProp, ImageInfoProp, ImagesProp, InfoProp, PageImagesProp};




pub struct ImagesRow<'a> {
    id: u32,
    title: &'a String,
    size: u128,
    width: usize,
    height: usize,
    url: &'a String,
    timestamp: &'a String,
    // category_titles: Vec<String>
}

pub struct CategoriesRow {
    id: u32,
    title: String,
    parent_category_titles: Vec<String>

}

pub struct ImageCategoryRow {
    image_id: u32,
    category_id: u32
}

pub struct SubCategoryRow {
    category_id: u32,
    subcategory_id: u32
}


pub fn into_images_row<'a>(query: &'a IndiscriminateQueryResult) -> crate::Result<ImagesRow> {
     if let Some(prop) = &query.imageinfo {
            if let Some(ii) = prop.0.iter().next()  {
                let size = ii.size.ok_or(crate::Error::TryFromResponseIntoRow)?;
                let width = ii.width.ok_or(crate::Error::TryFromResponseIntoRow)?;
                let height = ii.height.ok_or(crate::Error::TryFromResponseIntoRow)?;
                let url: &'a String = ii.url.as_ref().ok_or(crate::Error::TryFromResponseIntoRow)?;
                let timestamp: &'a String = ii.timestamp.as_ref().ok_or(crate::Error::TryFromResponseIntoRow)?;

                let id = query.pageid;
                let title: &'a String = &query.title;

                return Ok(ImagesRow { id, title, size, width, height, url, timestamp })
            
            }
        }
        Err(crate::Error::TryFromResponseIntoRow)
}