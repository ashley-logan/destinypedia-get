/*
   cat api.php\?action\=query\&titles\=File\:Dire_Taken_Concept_1.jpg\&prop\=imageinfo\&iiprop\=url\&format\=json
{"batchcomplete":"","query":{"normalized":[{"from":"File:Dire_Taken_Concept_1.jpg","to":"File:Dire Taken Concept 1.jpg"}],
"pages":{"50985":{"pageid":50985,"ns":6,"title":"File:Dire Taken Concept 1.jpg","imagerepository":"local",
"imageinfo":[{"url":"https://destiny.wiki.gallery/images/9/96/Dire_Taken_Concept_1.jpg","descriptionurl":"https://www.destinypedia.com/File:Dire_Taken_Concept_1.jpg","descriptionshorturl":"https://www.destinypedia.com/index.php?curid=50985"}]}}}}
*/

use crate::models::deserialize::response::ResponseTrait;
use crate::models::error;
use crate::models::serialize::{PARAMS, query};

use reqwest::{Client, Request};
use serde::de::DeserializeOwned;

static USER_AGENT: &str = "DESTINY_FETCHER";
static BASE: &str = "https://destinypedia/api.php";

fn get_client() -> Client {
    Client::builder().user_agent(USER_AGENT).build().unwrap()
}

pub async fn get<T: ResponseTrait + DeserializeOwned>(
    params: &mut PARAMS<query::Query>,
) -> error::Result<Vec<T>> {
    let client: Client = get_client();
    let mut more: bool = true; // more results to get
    let mut responses: Vec<T> = vec![];

    while more {
        let (client, r) = client.get(BASE).query(&params).build_split();
        let req: Request = r?;

        let resp: T = client.execute(req).await?.json().await?;

        if let Some((ck, cv)) = resp.get_continue_param() {
            params.set_continue(ck, cv);
        } else {
            more = false;
        }

        responses.push(resp);
    }

    Ok(responses)
}

// pub async fn get_indiscriminate(
//     params: PARAMS<query::Query>,
// ) -> error::Result<Vec<IndiscriminateResponse>> {
//     let client: Client = get_client();
//     let more: bool = true; // more results to get
//     let mut responses: Vec<IndiscriminateResponse> = vec![];

//     while more {
//         let (req, client): (Request, Client) = client.get(BASE).query(&params).build_split()?;
//         let resp: IndiscriminateResponse = client.execute(req).await?.json().await?;

//         responses.push(resp);

//         if let Some((ck, cv)) = resp.get_continue_param() {
//             params.set_cont(ck, cv);
//         } else {
//             more = false;
//         }
//     }

//     Ok(responses)
// }

// #[cfg(test)]
// mod tests {
//     use crate::{ErrorFormat, Format, PARAMS, query};

//     fn construct_validated_params() -> PARAMS<query::Query> {
//         PARAMS::new()
//             .list(query::List::AllImages)
//             .validate()
//             .unwrap()
//     }

//     #[tokio::test]
//     async fn test_fetch_json1() {
//         let params = construct_validated_params();
//     }
// }
