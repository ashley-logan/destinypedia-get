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


Sequential Requests async
|
get response bytes slice, pass to crossbeam_channel
|
Any worker takes slice and deserializes into Reponse
|
(possibly parallel) iterate and TryInto Row for each Result in Response
|
pass Row into channel/mpsc for Writer worker
|
prepare statement via prepare_cached
|
once 200-500 (depending on batch size) in memory, write to db




*/

use crate::models::NAMESPACE;
use crate::models::deserialize::response::IndiscriminateResponse;
use crate::models::{Generator, Limit, PARAMS, ParamsBuilder, Prop, Query, error::Result};
use reqwest::Client;

pub(crate) fn get_images_sync_params() -> Result<PARAMS<Query>> {
    let builder: ParamsBuilder<Query> = ParamsBuilder::new()
        .with_generator(Generator::allimages_with(None, Some(Limit::Num(20))))
        .with_props([Prop::ImageInfo])
        .with_extra("gaisort", "name")
        .with_extra("iiprop", "url|size|dimensions|timestamp");

    builder.build()
}

pub(crate) fn get_categories_sync_params() -> Result<PARAMS<Query>> {
    let builder: ParamsBuilder<Query> = ParamsBuilder::new()
        .with_generator(Generator::allcategories_with(
            None,
            None,
            None,
            Some(Limit::Num(20)),
        ))
        .with_props([Prop::Categories, Prop::CategoryInfo])
        .with_extra("cllimit", "max");

    builder.build()
}

pub(crate) fn get_pages_sync_params() -> Result<PARAMS<Query>> {
    let builder: ParamsBuilder<Query> = ParamsBuilder::new()
        .with_generator(Generator::allpages_with(
            Some([NAMESPACE::PAGE]),
            Some(Limit::Num(20)),
        ))
        .with_props([Prop::Info, Prop::PageImages]);

    builder.build()
}
// action=query&generator=allimages&gailimit=max&gaisort=name&prop=imageinfo&iiprop=url|size|dimensions|timestamp|canonicaltitle
// action=query&generator=allcategories&format=jsonfm&prop=categoryinfo|categories&cllimit=max

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::{Client, Response};

    static BASE: &str = "https://www.destinypedia.com/api.php";

    #[tokio::test]
    async fn test_allimages() {
        let params: PARAMS<Query> = get_images_sync_params().unwrap();

        let r: Response = Client::new().get(BASE).query(&params).send().await.unwrap();

        assert!(r.status().is_success())
    }

    #[tokio::test]
    async fn test_allcategories() {
        let params: PARAMS<Query> = get_categories_sync_params().unwrap();

        let r: Response = Client::new().get(BASE).query(&params).send().await.unwrap();

        assert!(r.status().is_success())
    }
    #[tokio::test]
    async fn test_allpages() {
        let params: PARAMS<Query> = get_pages_sync_params().unwrap();

        let r: Response = Client::new().get(BASE).query(&params).send().await.unwrap();

        assert!(r.status().is_success())
    }
}
