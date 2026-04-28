use crate::query;
/*
"batchcomplete": "",
    "continue": {
        "aicontinue": "((STEEL_MEDULLA~)).jpg",
        "continue": "-||"
    },
    "query": {
        "pages": {
            "14441": {
                "pageid": 14441,
                "ns": 0,
                "title": "Dreaming City"
            }
        },
        "allimages": [
            {
                "name": "'Act_on_Instinct'.png",
                "timestamp": "2024-07-02T13:45:06Z",
                "url": "https://destiny.wiki.gallery/images/7/76/%27Act_on_Instinct%27.png",
                "descriptionurl": "https://www.destinypedia.com/File:%27Act_on_Instinct%27.png",
                "descriptionshorturl": "https://www.destinypedia.com/index.php?curid=44354",
                "ns": 6,
                "title": "File:'Act on Instinct'.png"
            },
*/

use serde::Deserialize;
pub struct ListResponse {
    list: query::List,
    batchcomplete: String,
    cont: Option<Continue>,
}

struct Continue {
    cont: String,
    sub_cont: SubContinue,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum SubContinue {
    ApContinue(String),
    AiContinue(String),
    AcContinue(String),
}

#[derive(Deserialize)]
struct AllImages {
    name: String,
    timestamp: String,
    url: String,
    descriptionurl: String,
    descriptionshorturl: String,
    ns: u32,
    title: String,
}

#[derive(Deserialize)]
struct AllCategories {
    category: String,
}

#[derive(Deserialize)]
struct AllPages {
    pageid: u32,
    ns: u32,
    title: String,
}
