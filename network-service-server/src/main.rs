use anyhow::Ok;
mod collector;
use axum::Json;
use axum::extract::Path;
use axum::response::Html;
use axum::{Extension, Router, routing::get};
use serde::Serialize;
use sqlx::FromRow;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Read the .env file and obtain the database URL
    dotenv::dotenv()?;
    let db_url = std::env::var("DATABASE_URL")?;

    // Get a database connection pool
    let pool = sqlx::SqlitePool::connect(&db_url).await?;

    let handle = tokio::spawn(collector::data_collector(pool.clone()));

    let app = Router::new()
        .route("/", get(index))
        .route("/collector.html", get(collector))
        .route("/api/all", get(show_all))
        .route("/api/collectors", get(show_collectors))
        .route("/api/collector/{uuid}", get(collector_data))
        .layer(Extension(pool));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = tokio::net::TcpListener::bind(addr.to_string())
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    handle.await??; // Two question marks - we're unwrapping the task result, and the result from running the collector.

    Ok(())
}

#[derive(FromRow, Debug, Serialize)]
pub struct DataPoint {
    id: i32,
    collector_id: String,
    received: i64,
    total_memory: i64,
    used_memory: i64,
    average_cpu: f32,
}

pub async fn show_all(Extension(pool): Extension<sqlx::SqlitePool>) -> Json<Vec<DataPoint>> {
    let rows = sqlx::query_as::<_, DataPoint>("SELECT * FROM timeseries")
        .fetch_all(&pool)
        .await
        .unwrap();

    return Json(rows);
}

#[derive(FromRow, Debug, Serialize)]
pub struct Collector {
    id: i32,
    collector_id: String,
    last_seen: i64,
}

pub async fn show_collectors(Extension(pool): Extension<sqlx::SqlitePool>) -> Json<Vec<Collector>> {
    const SQL: &str = "SELECT
    DISTINCT(id) AS id,
    collector_id,
    (SELECT MAX(received) FROM timeseries WHERE collector_id = ts.collector_id) AS last_seen
    FROM timeseries ts";

    let result = sqlx::query_as::<_, Collector>(SQL)
        .fetch_all(&pool)
        .await
        .unwrap();

    return Json(result);
}

pub async fn collector_data(
    Extension(pool): Extension<sqlx::SqlitePool>,
    uuid: Path<String>,
) -> Json<Vec<DataPoint>> {
    let rows = sqlx::query_as::<_, DataPoint>(
        "SELECT * FROM timeseries WHERE collector_id = ? ORDER BY received",
    )
    .bind(uuid.as_str())
    .fetch_all(&pool)
    .await
    .unwrap();

    return Json(rows);
}

async fn index() -> Html<String> {
    let path = std::path::Path::new("./src/index.html");
    let content = tokio::fs::read_to_string(path).await.unwrap();

    return Html(content);
}
async fn collector() -> Html<String> {
    let path = std::path::Path::new("./src/collector.html");
    let content = tokio::fs::read_to_string(path).await.unwrap();

    return Html(content);
}
