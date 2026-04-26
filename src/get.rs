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

mod query {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    pub struct Query {
        titles: Vec<String>,
        prop: Option<Prop>,
        list: Option<List>,
    }

    #[derive(Debug, Serialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum Prop {
        Info,
        PageImages,
        Images,
        ImageInfo,
    }
    #[derive(Debug, Serialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum List {
        AllImages,
        AllLinks,
        AllPages,
    }
}

mod parse {
    use serde::Serialize;
    #[derive(Debug, Serialize)]
    pub struct Parse {
        pageid: u16,
        page: String,
        prop: Prop,
    }

    #[derive(Debug, Serialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum Prop {
        Images,
        TocData,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum Action {
    Query,
    Parse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum ErrorFormat {
    PlainText,
    WikiText,
    HTML,
    Raw,
    None,
    BC,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum Format {
    Json,
    JsonFm,
    None,
    Php,
    PhpFm,
    RawFm,
    Xml,
    XmlFm,
}

#[derive(Debug, Serialize)]
pub struct UrlParameters {
    action: Action,
    // https://www.destinypedia.com/api.php?action=query&prop=categoryinfo&format=jsonfm&titles=Category:Images_of_Oryx,_the_Taken_King
    format: Format,
    errorformat: ErrorFormat,
}

impl UrlParameters {}

fn fetch(pages: Vec<String>) -> reqwest::Result<Client> {
    let client: Client = Client::builder().user_agent(USER_AGENT).build()?;

    // for pg in &pages {
    //     client.get(BASE).query(&[])
    // }
    todo!()
}
