use axum::{response::Html, routing::get, Router};

async fn hello() -> Html<&'static str> {
    Html("<h1>hello, world</h1>")
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
