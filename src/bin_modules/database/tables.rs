use super::schema::*;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use serde::Serialize;

#[derive(Serialize, Queryable, Identifiable, Selectable, Debug, PartialEq)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = images)]
pub struct Images {
    pub id: i32,
    pub title: String,
    pub url: String,
    pub size: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub timestamp_: Option<chrono::NaiveDateTime>,
}

#[derive(Serialize, Queryable, Identifiable, Selectable, Debug, PartialEq)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = categories)]
pub struct Categories {
    pub id: i32,
    pub title: String,
    pub subcats: Option<i32>,
    pub files: Option<i32>,
}

#[derive(Queryable, Identifiable, Selectable, Debug, Associations, PartialEq)]
#[diesel(table_name = image_categories)]
#[diesel(belongs_to(Images, foreign_key = image_id))]
#[diesel(belongs_to(Categories, foreign_key = category_id))]
#[diesel(primary_key(image_id, category_id))]
pub struct ImageCategories {
    image_id: i32,
    category_id: i32,
}

#[derive(Queryable, Identifiable, Selectable, Debug, Associations, PartialEq)]
#[diesel(table_name = subcategories)]
#[diesel(belongs_to(Categories, foreign_key = subcategory_id, foreign_key = category_id))]
#[diesel(primary_key(category_id, subcategory_id))]
pub struct Subcategories {
    category_id: i32,
    subcategory_id: i32,
}
