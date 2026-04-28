use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct Parse {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contentmodel: Option<ContentModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pageid: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<String>,
    prop: Prop,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Prop {
    Text,
    Images,
    TocData,
}

#[derive(Debug, Serialize, PartialEq)]
pub(crate) enum ContentModel {
    GadgetDefinition,
    GraphJsonConfig,
    JsonJsonConfig,
    JsonSchema,
    MassMessageListContent,
    NewsletterContent,
    Scribunto,
    SecurePoll,
    #[serde(rename = "css")]
    Css,
    #[serde(rename = "flow-board")]
    FlowBoard,
    #[serde(rename = "javascript")]
    Javascript,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "sanitized-css")]
    SanitizedCss,
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "translate-messagebundle")]
    TranslateMessagebundle,
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "vue")]
    Vue,
    #[serde(rename = "wikitext")]
    WikiText,
}
