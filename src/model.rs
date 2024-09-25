use serde::Deserialize;
use time::OffsetDateTime;



#[derive(Deserialize, Debug)]
pub struct PublishForm {
    #[serde(with = "time::serde::iso8601")]
    pub published: OffsetDateTime,
    pub title: Option<String>,
    pub author: Option<String>,
    pub markdown_content: Option<String>,
}