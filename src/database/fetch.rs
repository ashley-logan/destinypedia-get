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
use crate::models::deserialize::Continue;
use crate::models::deserialize::QueryResponse;
use crate::models::{Generator, Limit, PARAMS, ParamsBuilder, Prop, Query, error::Result};

// action=query&generator=allimages&gailimit=max&gaisort=name&prop=imageinfo&iiprop=url|size|dimensions|timestamp|canonicaltitle
// action=query&generator=allcategories&format=jsonfm&prop=categoryinfo|categories&cllimit=max
