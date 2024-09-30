use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Display};
use time::OffsetDateTime;

#[derive(Serialize, Debug)]
pub enum ApiError {
    Markdown(crate::page::MarkdownError),
    Content(crate::page::ContentError),
    Database(DbError),
    Publish(PublishResponse),
    InvalidJson(String),
    Sqlx(String),
}

impl From<sqlx::Error> for ApiError {
    fn from(value: sqlx::Error) -> Self {
        ApiError::Sqlx(value.to_string())
    }
}

impl Error for ApiError {}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize, Debug)]
pub struct PublishForm {
    #[serde(with = "time::serde::iso8601")]
    pub published: OffsetDateTime,
    pub title: Option<String>,
    pub author: Option<String>,
    pub markdown_content: Option<String>,
    pub draft: Option<bool>,
}

#[derive(Debug)]
pub struct DbPage {
    pub slug: String,
    pub draft: Option<bool>,
    pub published: OffsetDateTime,
    pub title: Option<String>,
    pub last_updated: Option<OffsetDateTime>,
    pub author: Option<String>,
    pub markdown_content: Option<String>,
}

#[derive(Serialize, Debug)]
pub enum DbError {
    SlugExists(String),
}

impl From<PublishForm> for DbPage {
    fn from(value: PublishForm) -> Self {
        let formatted_date = value
            .published
            .format(&time::format_description::parse("[year]-[month]-[day]").unwrap())
            .unwrap();

        let title_or_empty = if let Some(title) = value.title.clone() {
            title
        } else {
            String::new()
        };

        DbPage {
            slug: slug::slugify(format!("{}-{}", formatted_date, title_or_empty)),
            draft: value.draft,
            published: value.published,
            title: value.title,
            last_updated: None,
            author: value.author,
            markdown_content: value.markdown_content,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct PublishResponse {
    pub page: crate::page::Page,
    pub errors: Vec<ApiError>,
}
