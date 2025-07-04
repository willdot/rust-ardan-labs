use axum::{Router, response::Html, routing::get};
use serde::Serialize;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/text", get(say_hello_text))
        .route("/html", get(say_hello_html))
        .route("/file", get(say_hello_html_file))
        .route("/json", get(hello_json));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = tokio::net::TcpListener::bind(addr.to_string())
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn say_hello_text() -> &'static str {
    return "Hello World!";
}

async fn say_hello_html() -> Html<&'static str> {
    const HTML: &str = include_str!("hello.html");
    return Html(HTML);
}

async fn say_hello_html_file() -> Html<String> {
    let path = std::path::Path::new("src/hello.html");
    let content = tokio::fs::read_to_string(path).await.unwrap();
    return Html(content);
}

#[derive(Serialize)]
struct HelloJson {
    message: String,
}

async fn hello_json() -> axum::Json<HelloJson> {
    let message = HelloJson {
        message: "hi JSON".to_string(),
    };

    return axum::Json(message);
}
