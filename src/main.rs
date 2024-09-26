use futures_util::TryStreamExt;
use sqlx::SqlitePool;
use warp::{reject::Rejection, Filter};

mod config;
mod model;
mod page;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();
    let config: config::Config =
        toml::from_str(&std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap())
            .unwrap();

    println!("{:#?}", config);

    let sqlite_url = format!("sqlite://{}", config.db.sqlite_file.display())
        .replace("\\", "/")
        .replace("//?/", "");

    let pool = SqlitePool::connect(&sqlite_url).await.unwrap();

    let mut stream = sqlx::query_as!(page::Page, "select * from blog").fetch(&pool);
    while let Some(row) = stream.try_next().await.unwrap() {
        println!("{:#?}", row.validate());
    }

    let post_publish = warp::post()
        .and(warp::path("publish"))
        .and(warp::body::form())
        .and_then(move |form: model::PublishForm| async move {
            let page = form.into_page();
            Ok::<_, Rejection>(format!("{:#?}", page))
        });

    warp::serve(post_publish.with(warp::log("asdf")))
        .run(config.net.bind)
        .await;
}
