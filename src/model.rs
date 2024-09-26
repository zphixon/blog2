use crate::page::Page;
use anyhow::Result;
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

impl PublishForm {
    pub fn into_page(self) -> Page {
        let formatted_date = self
            .published
            .format(&time::format_description::parse("[year]-[month]-[day]").unwrap())
            .unwrap();

        let title_or_empty = if let Some(title) = self.title.clone() {
            title
        } else {
            String::new()
        };

        Page {
            slug: Some(slug::slugify(format!(
                "{}-{}",
                formatted_date, title_or_empty
            ))),
            published: self.published,
            title: self.title,
            last_updated: None,
            author: self.author,
            markdown_content: self.markdown_content,
        }
    }
}
