use indexmap::IndexMap;
use serde::Serialize;
use time::OffsetDateTime;

#[derive(Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct Html(pub String);

#[derive(Serialize, Clone, Debug)]
pub struct Page {
    pub slug: String,
    #[serde(with = "time::serde::iso8601")]
    pub published: OffsetDateTime,
    pub draft: bool,
    pub title: Option<String>,
    #[serde(with = "time::serde::iso8601::option")]
    pub last_updated: Option<OffsetDateTime>,
    pub author: Option<String>,
    pub content: Html,
    pub linked_slugs: Vec<String>,
    pub footnotes: IndexMap<String, Html>,
}
