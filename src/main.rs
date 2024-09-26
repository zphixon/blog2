use futures_util::TryStreamExt;
use indexmap::IndexMap;
use serde::Serialize;
use sqlx::{Pool, SqlitePool};
use std::{fmt::Display, sync::Arc};
use tokio::sync::RwLock;
use warp::{
    http::{Response, StatusCode},
    reject::Rejection,
    Filter,
};

mod config;
mod model;
mod page;

fn four_hundred(err: impl Display) -> Response<String> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(format!("{}", err))
        .unwrap()
}

fn five_hundred(err: impl Display) -> Response<String> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(format!("{}", err))
        .unwrap()
}

macro_rules! try_or_500 {
    ($e:expr) => {
        try_or_500!($e, five_hundred)
    };

    ($e:expr, $wrap:ident) => {
        match $e.map_err($wrap) {
            Ok(ok) => ok,
            Err(err) => return err,
        }
    };
}

macro_rules! try_or_ok_500 {
    ($e:expr) => {
        try_or_ok_500!($e, five_hundred)
    };

    ($e:expr, $wrap:ident) => {
        match $e.map_err($wrap) {
            Ok(ok) => ok,
            Err(err) => return Ok(err),
        }
    };
}

async fn saturate(db_page: model::database::DbPage, state: State) -> Response<String> {
    let (page, errors) = db_page.saturate();
    let response_json = try_or_500!(serde_json::to_string(&model::network::PublishResponse {
        page: &page,
        errors,
    }));

    {
        state.cache.write().await.insert(page.slug.clone(), page);
    }

    Response::builder()
        .header("Content-Type", "application/json")
        .status(StatusCode::CREATED)
        .body(response_json)
        .unwrap()
}

async fn post_publish(db_page: model::database::DbPage, state: State) -> Response<String> {
    if state.cache.read().await.contains_key(&db_page.slug) {
        return four_hundred(format!("page with slug {} already exists", db_page.slug));
    }

    try_or_500!(
        sqlx::query!(
            "INSERT INTO blog (slug, draft, published, title, author, markdown_content) VALUES (?, ?, datetime(?), ?, ?, ?)",
            db_page.slug,
            db_page.draft,
            db_page.published,
            db_page.title,
            db_page.author,
            db_page.markdown_content,
        )
        .execute(&state.pool)
        .await
    );

    saturate(db_page, state).await
}

async fn put_publish(db_page: model::database::DbPage, state: State) -> Response<String> {
    if !state.cache.read().await.contains_key(&db_page.slug) {
        return post_publish(db_page, state).await;
    }

    try_or_500!(
        sqlx::query!(
            "UPDATE blog SET (draft, published, title, author, markdown_content) = (?, datetime(?), ?, ?, ?) WHERE slug = ?",
            db_page.draft,
            db_page.published,
            db_page.title,
            db_page.author,
            db_page.markdown_content,
            db_page.slug,
        )
        .execute(&state.pool)
        .await
    );

    saturate(db_page, state).await
}

#[derive(Clone)]
struct State {
    pool: SqlitePool,
    cache: Arc<RwLock<IndexMap<String, page::Page>>>,
}

// prefer auto-implemented Send and Sync
//unsafe impl Send for State {}
//unsafe impl Sync for State {}

fn with_state<S: Clone + Send + Sync>(
    state: S,
) -> impl Filter<Extract = (S,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

fn with_base_path(base_path: &'static str) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    if base_path.is_empty() {
        warp::any().boxed()
    } else {
        warp::path(base_path).boxed()
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    let config: &'static config::Config = Box::leak(Box::new(
        toml::from_str(&std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap())
            .unwrap(),
    ));
    println!("{:#?}", config);

    let sqlite_url = format!("sqlite://{}", config.db.sqlite_file.display())
        .replace("\\", "/")
        .replace("//?/", "");

    let pool = SqlitePool::connect(&sqlite_url).await.unwrap();
    let cache = Arc::new(RwLock::new(IndexMap::new()));

    {
        let mut stream =
            sqlx::query_as!(model::database::DbPage, "SELECT * FROM blog").fetch(&pool);
        while let Some(row) = stream.try_next().await.unwrap() {
            let slug = row.slug.clone();
            let (page, _) = row.saturate();
            println!("{:#?}", page);
            cache.write().await.insert(slug, page);
        }
    }

    let state = State { pool, cache };

    let post_publish_route = warp::post()
        .and(with_base_path(&config.net.base_path))
        .and(warp::path("publish"))
        .and(warp::body::form())
        .and(with_state(state.clone()))
        .and_then(|form: model::network::PublishForm, state| async {
            Ok::<_, Rejection>(post_publish(form.into_page(), state).await)
        });

    let put_publish_route = warp::put()
        .and(with_base_path(&config.net.base_path))
        .and(warp::path("publish"))
        .and(warp::body::form())
        .and(with_state(state.clone()))
        .and_then(|form: model::network::PublishForm, state| async {
            Ok::<_, Rejection>(put_publish(form.into_page(), state).await)
        });

    warp::serve(
        post_publish_route
            .or(put_publish_route)
            .with(warp::log("asdf")),
    )
    .run(config.net.bind)
    .await;
}
