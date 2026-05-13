/*
   cat api.php\?action\=query\&titles\=File\:Dire_Taken_Concept_1.jpg\&prop\=imageinfo\&iiprop\=url\&format\=json
{"batchcomplete":"","query":{"normalized":[{"from":"File:Dire_Taken_Concept_1.jpg","to":"File:Dire Taken Concept 1.jpg"}],
"pages":{"50985":{"pageid":50985,"ns":6,"title":"File:Dire Taken Concept 1.jpg","imagerepository":"local",
"imageinfo":[{"url":"https://destiny.wiki.gallery/images/9/96/Dire_Taken_Concept_1.jpg","descriptionurl":"https://www.destinypedia.com/File:Dire_Taken_Concept_1.jpg","descriptionshorturl":"https://www.destinypedia.com/index.php?curid=50985"}]}}}}
*/

use crate::serialize;
use crate::{Continue, Error, PARAMS, ParamsBuilder, Result, serialize::Query};
use crossbeam_channel::{Receiver, Sender, bounded};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use serde_json::from_slice;

const USER_AGENT: &'static str = "DESTINY_FETCHER";
const BASE: &'static str = "https://www.destinypedia.com/api.php";

fn get_client() -> Client {
    Client::builder().user_agent(USER_AGENT).build().unwrap()
}

pub async fn request_worker(params: &mut PARAMS<Query>, send: Sender<Vec<u8>>) -> Result<()> {
    let client: Client = get_client();
    let mut more_results: bool = true;

    while more_results {
        let resp: Response = client.get(BASE).query(&params).send().await?;

        let resp: Response = resp.error_for_status()?;

        let b: &[u8] = &resp.bytes().await?[..];

        send.send(b.to_vec());

        match from_slice::<Continue>(b).map(|c| c.into_tuple()) {
            Ok(Some((ck, cv))) => {
                params.update_continue(ck, cv);
            }
            _ => {
                more_results = false;
            }
        }
    }

    Ok(())
}

pub(crate) fn get_images_sync_params() -> Result<PARAMS<Query>> {
    let builder: ParamsBuilder<Query> = ParamsBuilder::new()
        .with_generator(serialize::Generator::allimages_with(
            None,
            Some(serialize::Limit::Max),
        ))
        .with_props([serialize::Prop::ImageInfo])
        .with_extra("gaisort", "name")
        .with_extra("iiprop", "url|size|dimensions|timestamp");

    builder.build()
}

pub(crate) fn get_categories_sync_params() -> Result<PARAMS<Query>> {
    let builder: ParamsBuilder<Query> = ParamsBuilder::new()
        .with_generator(serialize::Generator::allcategories_with(
            None,
            None,
            None,
            Some(serialize::Limit::Max),
        ))
        .with_props([serialize::Prop::Categories, serialize::Prop::CategoryInfo])
        .with_extra("cllimit", "max");

    builder.build()
}

pub(crate) fn get_pages_sync_params() -> Result<PARAMS<Query>> {
    let builder: ParamsBuilder<Query> = ParamsBuilder::new()
        .with_generator(serialize::Generator::allpages_with(
            Some([crate::models::NAMESPACE::PAGE]),
            Some(serialize::Limit::Max),
        ))
        .with_props([serialize::Prop::Info, serialize::Prop::PageImages]);

    builder.build()
}

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
