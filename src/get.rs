/*
   cat api.php\?action\=query\&titles\=File\:Dire_Taken_Concept_1.jpg\&prop\=imageinfo\&iiprop\=url\&format\=json
{"batchcomplete":"","query":{"normalized":[{"from":"File:Dire_Taken_Concept_1.jpg","to":"File:Dire Taken Concept 1.jpg"}],
"pages":{"50985":{"pageid":50985,"ns":6,"title":"File:Dire Taken Concept 1.jpg","imagerepository":"local",
"imageinfo":[{"url":"https://destiny.wiki.gallery/images/9/96/Dire_Taken_Concept_1.jpg","descriptionurl":"https://www.destinypedia.com/File:Dire_Taken_Concept_1.jpg","descriptionshorturl":"https://www.destinypedia.com/index.php?curid=50985"}]}}}}
*/

use reqwest::{Client, Request};
use serde::{Deserialize, Serialize};

static USER_AGENT: &str = "DP_FETCHER";
static BASE: &str = "https://destinypedia/api.php";

fn fetch(pages: Vec<String>) -> reqwest::Result<Client> {
    let client: Client = Client::builder().user_agent(USER_AGENT).build()?;

    // for pg in &pages {
    //     client.get(BASE).query(&[])
    // }
    todo!()
}
