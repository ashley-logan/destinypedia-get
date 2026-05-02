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

// action=query&generator=allimages&gailimit=max&gaisort=name&gaiprop=url|size|timestamp&prop=imageinfo&iiprop=url|size|dimensions|timestamp
async fn fetch_images(gaicontinue: String) {
    let params = json!({
        "action": "query",
        "format": "json",
        "generator": "allimages",
        "gailimit": "max",
        "gaisort": "name",
        "prop": "imageinfo",
        "iiprop": "url|size|dimensions|timestamp",
        "gaicontinue": gaicontinue
    });

    todo!("loop through generator")
}
// action=query&generator=allcategories&format=jsonfm&prop=categoryinfo|categories&cllimit=max
async fn fetch_categories(accontinue: String) {
    let params = json!({
        "action": "query",
        "format": "json",
        "generator": "allcategories",
        "gaclimit": "max",
        "prop": "categoryinfo|categories",
        "cllimit": "max",
        "accontinue": accontinue
    });

    todo!("loop through generator")
}

async fn fetch_pages(gapcontinue: String) {
    let params = json!({
        "action": "query",
        "format": "json",
        "generator": "allpages",
        "gapnamespace": 0,
        "gaplimit": "max",
        "prop": "info"
    });
}
