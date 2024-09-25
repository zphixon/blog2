#[tokio::main(flavor = "current_thread")]
async fn main() {
    let db_path = std::env::var("SCHEMA_DB_FILE").unwrap();
    if !tokio::fs::try_exists(&db_path).await.unwrap() {
        tokio::fs::write(&db_path, "").await.unwrap();
    }

    let schema = include_str!("schema.sql");
    let sqlite_url = std::env::var("DATABASE_URL").unwrap();
    let pool = sqlx::SqlitePool::connect(&sqlite_url).await.unwrap();
    sqlx::query(schema).execute(&pool).await.unwrap();
}
