// @generated automatically by Diesel CLI.

diesel::table! {
    categories (id) {
        id -> Nullable<Integer>,
        title -> Text,
        subcats -> Nullable<Integer>,
        files -> Nullable<Integer>,
    }
}

diesel::table! {
    image_categories (image_id, category_id) {
        image_id -> Integer,
        category_id -> Integer,
    }
}

diesel::table! {
    images (id) {
        id -> Nullable<Integer>,
        title -> Text,
        url -> Text,
        size -> Nullable<Integer>,
        width -> Nullable<Integer>,
        height -> Nullable<Integer>,
        timestamp_ -> Nullable<Timestamp>,
    }
}

diesel::table! {
    subcategories (category_id, subcategory_id) {
        category_id -> Integer,
        subcategory_id -> Integer,
    }
}

diesel::joinable!(image_categories -> categories (category_id));
diesel::joinable!(image_categories -> images (image_id));

diesel::allow_tables_to_appear_in_same_query!(categories, image_categories, images, subcategories,);
