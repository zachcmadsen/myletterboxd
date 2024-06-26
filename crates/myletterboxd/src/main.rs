use std::str::FromStr;

use axum::{response::Html, routing::get, Router};
use sqlx::{
    pool::PoolOptions,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};

async fn hello() -> Html<&'static str> {
    Html("<h1>hello, world</h1>")
}

#[tokio::main]
async fn main() {
    let x = SqliteConnectOptions::from_str("sqlite::memory:")
        .unwrap()
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);
    let pool = SqlitePool::connect_with(x).await.unwrap();

    sqlx::query(
        r#"
        CREATE TABLE todos (
            id INTEGER PRIMARY KEY,
            description TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let id = sqlx::query("INSERT INTO todos (description) VALUES (?1)")
        .bind("cool description")
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();
    println!("row id: {}", id);

    let app = Router::new().route("/", get(hello));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
