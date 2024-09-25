use futures_util::TryStreamExt;
use pulldown_cmark::{BrokenLinkCallback, Event, LinkType, Options, Parser, Tag};
use sqlx::SqlitePool;
use warp::Filter;

mod config;
mod model;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let config: config::Config =
        toml::from_str(&std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap())
            .unwrap();

    println!("{:#?}", config);

    let sqlite_url = format!("sqlite://{}", config.db.sqlite_file.display());
    let pool = SqlitePool::connect(&sqlite_url).await.unwrap();

    let mut stream = sqlx::query!("select * from blog").fetch(&pool);
    while let Some(row) = stream.try_next().await.unwrap() {
        println!("{:?}", row);

        if let Some(content) = row.markdown_content.as_ref() {
            let parser = Parser::new_with_broken_link_callback(
                content,
                Options::all(),
                Some(|link| {
                    println!("broken link? {:?}", link);
                    None
                }),
            )
            .filter_map(|event| match event {
                Event::Start(Tag::Link {
                    link_type: LinkType::Autolink | LinkType::Inline,
                    ref dest_url,
                    ref title,
                    ref id,
                }) => {
                    println!("link to {}", dest_url);
                    Some(event)
                },

                Event::Start(Tag::FootnoteDefinition(footnote)) => {
                    println!("footnote {}", footnote);
                    None
                },

                // _ if in_footnote => { footnotes.push(event); None }
                _ => Some(event),
            });

            let mut html = String::new();
            pulldown_cmark::html::push_html(&mut html, parser);

            println!("{}", html);
        }
    }

    warp::post()
        .and(warp::path("publish"))
        .and(warp::body::form())
        .map(|_form: model::PublishForm| {});
}
