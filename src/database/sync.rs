use serde_json::json;
/*
DATABASE SCHEMA
    IMAGES
        id, name, size, width, height, url, timestamp
    IMAGE_CATEGORIES
        image_id, category_id
    CATEGORIES
        id, name, size,

    SUBCATEGORIES
        category_id, subcategory_id

    PAGES
        id, name
    PAGE_CATEGORIES
        page_id, category_id
    PAGE_IMAGES
        page_id, image_id

    maybe: GRIMOIRE
*/

async fn fetch_images() {
    let params = json!({
        "action": "query",
        "generator": "allimages",
        "gailimit": "max",
        
    })
}