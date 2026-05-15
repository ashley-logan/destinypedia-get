/*
   cat api.php\?action\=query\&titles\=File\:Dire_Taken_Concept_1.jpg\&prop\=imageinfo\&iiprop\=url\&format\=json
{"batchcomplete":"","query":{"normalized":[{"from":"File:Dire_Taken_Concept_1.jpg","to":"File:Dire Taken Concept 1.jpg"}],
"pages":{"50985":{"pageid":50985,"ns":6,"title":"File:Dire Taken Concept 1.jpg","imagerepository":"local",
"imageinfo":[{"url":"https://destiny.wiki.gallery/images/9/96/Dire_Taken_Concept_1.jpg","descriptionurl":"https://www.destinypedia.com/File:Dire_Taken_Concept_1.jpg","descriptionshorturl":"https://www.destinypedia.com/index.php?curid=50985"}]}}}}
*/

use crate::{
    GcmIdentifier, Generator, Limit, PARAMS, ParamsBuilder, Prop, Query, Result, models::NAMESPACE,
};

pub(crate) fn get_images_sync_params() -> Result<PARAMS<Query>> {
    let builder: ParamsBuilder<Query> = ParamsBuilder::new()
        .with_generator(Generator::allimages_with(None, Some(Limit::Max)))
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
            Some(Limit::Max),
        ))
        .with_props([Prop::Categories, Prop::CategoryInfo])
        .with_extra("cllimit", "max");

    builder.build()
}

pub(crate) fn get_category_members_sync_params(pageid: u32) -> Result<PARAMS<Query>> {
    let builder: ParamsBuilder<Query> = ParamsBuilder::new()
        .with_generator(Generator::categorymembers_with(
            GcmIdentifier::GcmPageid(pageid),
            Some(vec![NAMESPACE::CATEGORY, NAMESPACE::FILE]),
            Some(Limit::Max),
        ))
        .with_props([Prop::ImageInfo, Prop::CategoryInfo])
        .with_extra("iiprop", "url|timestamp|dimensions|size|canonicaltitle");
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QueryResponse;
    use reqwest::{Client, Response};

    static BASE: &str = "https://www.destinypedia.com/api.php";

    #[tokio::test]
    async fn test_allimages() {
        let params: PARAMS<Query> = get_images_sync_params().unwrap();

        let r: Response = Client::new().get(BASE).query(&params).send().await.unwrap();

        assert!(r.status().is_success());

        let _json: QueryResponse = r.json().await.unwrap();
    }

    #[tokio::test]
    async fn test_allcategories() {
        let params: PARAMS<Query> = get_categories_sync_params().unwrap();

        let r: Response = Client::new().get(BASE).query(&params).send().await.unwrap();

        assert!(r.status().is_success());

        let _json: QueryResponse = r.json().await.unwrap();
    }

    #[tokio::test]
    async fn test_category_members() {
        let params: PARAMS<Query> = get_category_members_sync_params(363_u32).unwrap();

        let r = Client::new().get(BASE).query(&params).send().await.unwrap();
        assert!(r.status().is_success());

        let _json: QueryResponse = r.json().await.unwrap();
    }
}
