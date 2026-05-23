use diesel::prelude::*;

use super::schema::*;

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq)]
#[diesel(table_name = images)]
pub struct Images {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub size: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub timestamp: Option<String>,
}

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq)]
#[diesel(table_name = categories)]
pub struct Categories {
    pub id: u32,
    pub title: String,
    pub subcats: Option<u32>,
    pub files: Option<u32>,
}

#[derive(Queryable, Identifiable, Selectable, Debug, Associations, PartialEq)]
#[diesel(table_name = image_categories)]
#[diesel(belongs_to(Images, foreign_key = image_id))]
#[diesel(belongs_to(Categories, foreign_key = category_id))]
#[diesel(primary_key(image_id, category_id))]
pub struct ImageCategories {
    image_id: u32,
    category_id: u32,
}

#[derive(Queryable, Identifiable, Selectable, Debug, Associations, PartialEq)]
#[diesel(table_name = subcategories)]
#[diesel(belongs_to(Categories, foreign_key = subcategory_id, foreign_key = category_id))]
#[diesel(primary_key(category_id, subcategory_id))]
pub struct Subcategories {
    category_id: u32,
    subcategory_id: u32,
}
