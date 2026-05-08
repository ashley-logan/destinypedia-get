use derive_more::Display;

#[derive(Debug, Display, derive_more::PartialEq, derive_more::Eq)]
#[display(rename_all = "lowercase")]
pub enum ImageInfoProp {
    Timestamp,
    User,
    Userid,
    Comment,
    Parsedcomment,
    Canonicaltitle,
    Url,
    Size,
    Dimensions,
    SHA1,
    Mime,
    Mediatype,
    Metadata,
    Commonmetadata,
    Extmetadata,
}

#[derive(Debug, derive_more::PartialEq, derive_more::Eq, Display)]
#[display(rename_all = "lowercase")]
pub enum Prop {
    Info,
    PageImages,
    Images,
    ImageInfo,
    Categories,
    CategoryInfo,
    FileUsage,
}

#[derive(Debug, Display, derive_more::PartialEq, derive_more::Eq)]
#[display(rename_all = "lowercase")]
pub enum CategoryProp {
    Ids,
    Title,
    Sortkey,
    Sortkeyprefix,
    Type,
    Timestamp,
}
