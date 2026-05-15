/*
   cat api.php\?action\=query\&titles\=File\:Dire_Taken_Concept_1.jpg\&prop\=imageinfo\&iiprop\=url\&format\=json
{"batchcomplete":"","query":{"normalized":[{"from":"File:Dire_Taken_Concept_1.jpg","to":"File:Dire Taken Concept 1.jpg"}],
"pages":{"50985":{"pageid":50985,"ns":6,"title":"File:Dire Taken Concept 1.jpg","imagerepository":"local",
"imageinfo":[{"url":"https://destiny.wiki.gallery/images/9/96/Dire_Taken_Concept_1.jpg","descriptionurl":"https://www.destinypedia.com/File:Dire_Taken_Concept_1.jpg","descriptionshorturl":"https://www.destinypedia.com/index.php?curid=50985"}]}}}}
*/

use crate::serialize;
use crate::{
    Continue, Error, PARAMS, ParamsBuilder, Prop, Result, error::SendErrorGeneric,
    models::NAMESPACE, serialize::Query,
};
use crossbeam_channel::{Receiver, Sender, bounded};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use serde_json::from_slice;

const USER_AGENT: &'static str = "DESTINY_FETCHER";
const BASE: &'static str = "https://www.destinypedia.com/api.php";

fn get_client() -> Client {
    Client::builder().user_agent(USER_AGENT).build().unwrap()
}

/// intial queries are built and send into the params channel
/// producer pulls a query from the channel
/// producer calls a request, and checks if the response container a continue value (signifying more results)
/// producer sends response bytes to consumer
/// consumer deserializes the response into a QueryResponse
/// consumer processes response and transforms into Row(s)
/// In some cases, consumer constructs a new query and sends it into the params channel
/// consumer sends Row(s) into row channel
/// writer recieves rows and prepares an insert statement
/// once batch size is reached (or on final flush) writer bulk writes to the db

pub async fn category_members_params_producer(
    recv: Receiver<u32>,
    send: Sender<PARAMS<Query>>,
) -> Result<usize> {
    let mut num_requests: usize = 0;
    while let Ok(id) = recv.recv() {
        let params: PARAMS<Query> = get_category_members_sync_params(id)?;
        send.send(params)
            .expect("failed to send query parameters to cm_request_worker");
        num_requests += 1;
    }
    Ok(num_requests)
}

pub async fn cm_request_worker(recv: Receiver<u32>, send: Sender<(u32, Vec<u8>)>) -> Result<()> {
    let client: Client = get_client();

    while let Ok(id) = recv.recv() {
        let mut params: PARAMS<Query> = get_category_members_sync_params(id.clone())?;
        let mut more_results: bool = true;

        while more_results {
            let resp: Response = client.get(BASE).query(&params).send().await?;

            resp.error_for_status_ref()?;

            let b: Vec<u8> = resp.bytes().await?.to_vec();

            match from_slice::<Continue>(&b).map(|c| c.into_tuple()) {
                Ok(Some((ck, cv))) => {
                    params.update_continue(ck, cv);
                }
                _ => {
                    more_results = false;
                }
            }

            send.send((id, b)).unwrap(); // send owned bytes into producer-consumer channel
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

pub(crate) fn get_category_members_sync_params(pageid: u32) -> Result<PARAMS<Query>> {
    let builder: ParamsBuilder<Query> = ParamsBuilder::new()
        .with_generator(serialize::Generator::categorymembers_with(
            serialize::GcmIdentifier::GcmPageid(pageid),
            Some(vec![NAMESPACE::CATEGORY, NAMESPACE::FILE]),
            Some(serialize::Limit::Max),
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
