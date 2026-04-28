/*
   cat api.php\?action\=query\&titles\=File\:Dire_Taken_Concept_1.jpg\&prop\=imageinfo\&iiprop\=url\&format\=json
{"batchcomplete":"","query":{"normalized":[{"from":"File:Dire_Taken_Concept_1.jpg","to":"File:Dire Taken Concept 1.jpg"}],
"pages":{"50985":{"pageid":50985,"ns":6,"title":"File:Dire Taken Concept 1.jpg","imagerepository":"local",
"imageinfo":[{"url":"https://destiny.wiki.gallery/images/9/96/Dire_Taken_Concept_1.jpg","descriptionurl":"https://www.destinypedia.com/File:Dire_Taken_Concept_1.jpg","descriptionshorturl":"https://www.destinypedia.com/index.php?curid=50985"}]}}}}
*/

use crate::{PARAMS, query};
use reqwest::Client;
use serde::{Deserialize, Serialize};

static USER_AGENT: &str = "DESTINY_FETCHER";
static BASE: &str = "https://destinypedia/api.php";

async fn fetch_json(params: PARAMS<query::Query>) -> reqwest::Result<Client> {
    let client: Client = Client::builder().user_agent(USER_AGENT).build()?;

    // for pg in &pages {
    //     client.get(BASE).query(&[])
    // }
    let resp = client.get(BASE).query(&params).send().await?;

    todo!()
}

#[cfg(test)]
mod tests {
    use crate::{ErrorFormat, Format, PARAMS, query};

    fn construct_validated_params() -> PARAMS<query::Query> {
        PARAMS::new()
            .list(query::List::AllImages)
            .validate()
            .unwrap()
    }

    #[tokio::test]
    async fn test_fetch_json1() {}
}
