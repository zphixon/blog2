use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Deserialize, Debug)]
pub struct PublishForm {
    #[serde(with = "time::serde::iso8601")]
    pub published: OffsetDateTime,
    pub title: Option<String>,
    pub author: Option<String>,
    pub markdown_content: Option<String>,
    pub draft: Option<bool>,
}

impl PublishForm {
    pub fn into_page(self) -> crate::model::database::DbPage {
        let formatted_date = self
            .published
            .format(&time::format_description::parse("[year]-[month]-[day]").unwrap())
            .unwrap();

        let title_or_empty = if let Some(title) = self.title.clone() {
            title
        } else {
            String::new()
        };

        crate::model::database::DbPage {
            slug: slug::slugify(format!("{}-{}", formatted_date, title_or_empty)),
            draft: self.draft,
            published: self.published,
            title: self.title,
            last_updated: None,
            author: self.author,
            markdown_content: self.markdown_content,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct PublishResponse<'page> {
    pub page: &'page crate::page::Page,
    pub errors: Vec<crate::model::database::ValidateError>,
}
