use indexmap::{IndexMap, IndexSet};
use pulldown_cmark::{BrokenLink, CowStr, Event, LinkType, Options, Parser, Tag, TagEnd};
use serde::Serialize;
use time::OffsetDateTime;

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
pub enum ValidateError {
    UnknownFootnote(String),
    UnreferencedFootnote(String),
    UnknownLink(String),
}

impl DbPage {
    pub fn saturate(self) -> (crate::page::Page, Vec<ValidateError>) {
        let mut errors = Vec::new();

        let mut content = String::new();
        let mut linked_slugs = Vec::new();

        let mut footnote_references = IndexSet::new();
        let mut in_footnotes = Vec::new();
        let mut footnote_events = IndexMap::new();

        let parser = Parser::new_with_broken_link_callback(
            self.markdown_content
                .as_ref()
                .map(String::as_str)
                .unwrap_or(""),
            Options::all(),
            Some(|BrokenLink { reference, .. }| Some((reference, CowStr::Borrowed("")))),
        )
        .filter_map(|event| match event {
            Event::Start(Tag::Link {
                link_type,
                ref dest_url,
                ..
            }) => {
                if dest_url.starts_with("@") {
                    linked_slugs.push(dest_url[1..].to_string());
                } else if matches!(
                    link_type,
                    LinkType::CollapsedUnknown
                        | LinkType::ReferenceUnknown
                        | LinkType::ShortcutUnknown
                ) {
                    errors.push(ValidateError::UnknownLink(dest_url.to_string()));
                }

                Some(event)
            }

            Event::FootnoteReference(ref name) => {
                footnote_references.insert(name.to_string());
                Some(event)
            }

            _ => Some(event),
        })
        .filter_map(|event| match event {
            Event::Start(Tag::FootnoteDefinition(ref name)) => {
                in_footnotes.push((Some(name.to_string()), vec![event]));
                None
            }

            Event::End(TagEnd::FootnoteDefinition) => {
                let (name, mut footnote) = in_footnotes.pop().unwrap();
                footnote.push(event);
                footnote_events.insert(name.unwrap(), footnote);
                None
            }

            _ if !in_footnotes.is_empty() => {
                in_footnotes.last_mut().unwrap().1.push(event);
                None
            }

            _ => Some(event),
        });

        pulldown_cmark::html::push_html(&mut content, parser);

        let mut footnotes = IndexMap::new();
        for reference in footnote_references {
            let Some(events) = footnote_events.shift_remove(&reference) else {
                errors.push(ValidateError::UnknownFootnote(reference));
                continue;
            };

            // already collected links from footnote markdown
            let mut footnote_html = String::new();
            pulldown_cmark::html::push_html(&mut footnote_html, events.into_iter());
            footnotes.insert(reference, crate::page::Html(footnote_html));
        }

        for (name, events) in footnote_events {
            errors.push(ValidateError::UnreferencedFootnote(name.clone()));

            // already collected links from footnote markdown
            let mut footnote_html = String::new();
            pulldown_cmark::html::push_html(&mut footnote_html, events.into_iter());
            footnotes.insert(name, crate::page::Html(footnote_html));
        }

        let page = crate::page::Page {
            slug: self.slug,
            published: self.published,
            draft: self.draft.unwrap_or(false),
            title: self.title,
            last_updated: self.last_updated,
            author: self.author,
            content: crate::page::Html(content),
            linked_slugs,
            footnotes,
        };

        (page, errors)
    }
}
