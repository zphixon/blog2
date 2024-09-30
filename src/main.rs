use axum::{extract::State as AxumState, Router};
use futures_util::TryStreamExt;
use indexmap::IndexMap;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::{net::TcpListener, sync::RwLock};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod compat;
mod config;
mod model;
mod page;

async fn ensure_slug_not_cached(
    state: State,
    db_page: &model::DbPage,
) -> Result<(), model::ApiError> {
    if state.cache.read().await.contains_key(&db_page.slug) {
        return Err(model::ApiError::Database(model::DbError::SlugExists(
            db_page.slug.clone(),
        )));
    }

    Ok(())
}

//async fn saturate(
//    db_page: model::DbPage,
//    state: State,
//    do_cache: bool,
//) -> Result<model::ApiError, model::ApiError> {
//    if do_cache {
//        ensure_slug_not_cached(state.clone(), &db_page).await?;
//    }
//
//    let (page, errors) = db_page.into();
//
//    let mut errors: Vec<model::ApiError> =
//        errors.into_iter().map(model::ApiError::Markdown).collect();
//    for linked_slug in page.linked_slugs.iter() {
//        if !state.cache.read().await.contains_key(linked_slug) {
//            errors.push(model::ApiError::Content(page::ContentError::UnknownSlug(
//                linked_slug.clone(),
//            )));
//        }
//    }
//
//    let response = model::ApiError::Publish(model::PublishResponse {
//        page: page.clone(),
//        errors,
//    });
//
//    if do_cache {
//        state.cache.write().await.insert(page.slug.clone(), page);
//    }
//
//    Ok(response)
//}
//
//async fn post_publish(form: String, state: State) -> Result<model::ApiError, model::ApiError> {
//    let form = serde_urlencoded::from_str::<model::PublishForm>(&form)?;
//    let db_page = form.into();
//
//    ensure_slug_not_cached(state.clone(), &db_page).await?;
//
//    sqlx::query!(
//        "INSERT INTO blog (slug, draft, published, title, author, markdown_content) VALUES (?, ?, datetime(?), ?, ?, ?)",
//        db_page.slug,
//        db_page.draft,
//        db_page.published,
//        db_page.title,
//        db_page.author,
//        db_page.markdown_content,
//    )
//    .execute(&state.pool)
//    .await?;
//
//    saturate(db_page, state, true).await
//}
//
//async fn put_publish(form_str: String, state: State) -> Result<model::ApiError, model::ApiError> {
//    let form = serde_urlencoded::from_str::<model::PublishForm>(&form_str)?;
//    let db_page: model::DbPage = form.into();
//
//    if !state.cache.read().await.contains_key(&db_page.slug) {
//        return post_publish(form_str, state).await;
//    }
//
//    sqlx::query!(
//        "UPDATE blog SET (draft, published, title, author, markdown_content) = (?, datetime(?), ?, ?, ?) WHERE slug = ?",
//        db_page.draft,
//        db_page.published,
//        db_page.title,
//        db_page.author,
//        db_page.markdown_content,
//        db_page.slug,
//    )
//    .execute(&state.pool)
//    .await?;
//
//    saturate(db_page, state, true).await
//}

#[derive(Clone)]
struct State {
    pool: SqlitePool,
    cache: Arc<RwLock<IndexMap<String, page::Page>>>,
}

async fn publish_post_handler(
    AxumState(state): AxumState<State>,
    compat::MyForm(form): compat::MyForm<model::PublishForm>,
) -> Result<(), ()> {
    tracing::debug!("{:?}", form);
    Ok(())
}

#[tokio::main()]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config: &'static config::Config = Box::leak(Box::new(
        toml::from_str(&std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap())
            .unwrap(),
    ));
    tracing::info!("{:#?}", config);

    let sqlite_url = format!("sqlite://{}", config.db.sqlite_file.display())
        .replace("\\", "/")
        .replace("//?/", "");

    let pool = SqlitePool::connect(&sqlite_url).await.unwrap();
    let cache = Arc::new(RwLock::new(IndexMap::new()));

    {
        let mut stream = sqlx::query_as!(model::DbPage, "SELECT * FROM blog").fetch(&pool);
        while let Some(row) = stream.try_next().await.unwrap() {
            let slug = row.slug.clone();
            let (page, _) = row.into();
            tracing::debug!("{:#?}", page);
            cache.write().await.insert(slug, page);
        }
    }

    let state = State { pool, cache };

    let app = Router::new()
        .route(
            &format!("{}/publish", config.net.url.path()),
            axum::routing::post(publish_post_handler),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = TcpListener::bind(config.net.bind).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
