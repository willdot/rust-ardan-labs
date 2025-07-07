use axum::Form;
use axum::Json;
use axum::body::Body;
use axum::extract::Multipart;
use axum::extract::Path;
use axum::http::header;
use axum::response::IntoResponse;
use axum::{Extension, Router, response::Html, routing::get, routing::post};
use futures::TryStreamExt;
use serde::Deserialize;
use serde::Serialize;
use sqlx::Sqlite;
use sqlx::{FromRow, Pool, Row};
use std::net::SocketAddr;
use tokio::task::spawn_blocking;
use tokio_util::io::ReaderStream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Read the .env file and obtain the database URL
    dotenv::dotenv()?;
    let db_url = std::env::var("DATABASE_URL")?;

    // Get a database connection pool
    let pool = sqlx::SqlitePool::connect(&db_url).await?;

    // Run Migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    // catchup on missing thumbnails
    fill_missing_thumbnails(&pool).await?;

    let app = Router::new()
        .route("/test", get(test))
        .route("/", get(index_page))
        .route("/upload", post(uploader))
        .route("/image/{id}", get(get_image))
        .route("/thumb/{id}", get(get_thumbnail))
        .route("/images", get(list_images))
        .route("/search", post(search_images))
        .layer(Extension(pool));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = tokio::net::TcpListener::bind(addr.to_string())
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn test(Extension(pool): Extension<sqlx::SqlitePool>) -> String {
    let result = sqlx::query("SELECT COUNT(id) FROM images")
        .fetch_one(&pool)
        .await
        .unwrap();
    let count = result.get::<i64, _>(0);
    format!("{count} images in the database")
}

async fn index_page() -> Html<String> {
    let path = std::path::Path::new("./index.html");
    let content = tokio::fs::read_to_string(path).await.unwrap();
    Html(content)
}

async fn uploader(
    Extension(pool): Extension<sqlx::SqlitePool>,
    mut multipart: Multipart,
) -> Html<String> {
    let mut tags: Option<String> = None;
    let mut image: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!("{name} is {} bytes", data.len());

        match name.as_str() {
            "tags" => tags = Some(String::from_utf8(data.to_vec()).unwrap()),
            "image" => image = Some(data.to_vec()),
            _ => panic!("unknown field: {name}"),
        }
    }

    if let (Some(tags), Some(image)) = (tags, image) {
        let new_image_id = insert_image_into_database(&pool, &tags).await.unwrap();
        save_image(new_image_id, &image).await.unwrap();
        spawn_blocking(move || make_thumbnail(new_image_id).unwrap());
        println!("{new_image_id}");
    } else {
        panic!("Missing field")
    }

    let path = std::path::Path::new("./redirect.html");
    let content = tokio::fs::read_to_string(path).await.unwrap();

    return Html(content);
}

async fn insert_image_into_database(pool: &sqlx::SqlitePool, tags: &str) -> anyhow::Result<i64> {
    let row = sqlx::query("INSERT INTO images (tags) VALUES (?) RETURNING id")
        .bind(tags)
        .fetch_one(pool)
        .await?;

    Ok(row.get(0))
}

async fn save_image(id: i64, bytes: &[u8]) -> anyhow::Result<()> {
    let base_path = std::path::Path::new("images");
    if !base_path.exists() || !base_path.is_dir() {
        tokio::fs::create_dir_all(base_path).await?
    }

    let image_path = base_path.join(format!("{id}.jpg"));
    if image_path.exists() {
        anyhow::bail!("File already exists");
    }

    tokio::fs::write(image_path, bytes).await?;

    Ok(())
}

async fn get_image(Path(id): Path<i64>) -> impl IntoResponse {
    let filename = format!("images/{id}.jpg");
    let attachment = format!("filename={filename}");

    let file = tokio::fs::File::open(&filename).await.unwrap();
    let stream = ReaderStream::new(file);

    axum::response::Response::builder()
        .header(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("image/jpeg"),
        )
        .header(
            header::CONTENT_DISPOSITION,
            header::HeaderValue::from_str(&attachment).unwrap(),
        )
        .body(Body::from_stream(stream))
        .unwrap()
}

async fn get_thumbnail(Path(id): Path<i64>) -> impl IntoResponse {
    let filename = format!("images/{id}_thumb.jpg");
    let attachment = format!("filename={filename}");

    let file = tokio::fs::File::open(&filename).await.unwrap();
    let stream = ReaderStream::new(file);

    axum::response::Response::builder()
        .header(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("image/jpeg"),
        )
        .header(
            header::CONTENT_DISPOSITION,
            header::HeaderValue::from_str(&attachment).unwrap(),
        )
        .body(Body::from_stream(stream))
        .unwrap()
}

fn make_thumbnail(id: i64) -> anyhow::Result<()> {
    let image_path = format!("images/{id}.jpg");
    let thumbnail_path = format!("images/{id}_thumb.jpg");
    let image_bytes: Vec<u8> = std::fs::read(image_path)?;

    let image = if let Ok(format) = image::guess_format(&image_bytes) {
        image::load_from_memory_with_format(&image_bytes, format)?
    } else {
        image::load_from_memory(&image_bytes)?
    };
    let thumbnail = image.thumbnail(100, 100);
    thumbnail.save(thumbnail_path)?;
    Ok(())
}

async fn fill_missing_thumbnails(pool: &Pool<Sqlite>) -> anyhow::Result<()> {
    let mut rows = sqlx::query("SELECT id FROM images").fetch(pool);

    while let Some(row) = rows.try_next().await? {
        let id = row.get::<i64, _>(0);
        let thumbnail_path = format!("images/{id}_thumb.jpg");
        if !std::path::Path::new(&thumbnail_path).exists() {
            spawn_blocking(move || make_thumbnail(id)).await??;
        }
    }

    Ok(())
}

#[derive(Deserialize, Serialize, FromRow, Debug)]
struct ImageRecord {
    id: i64,
    tags: String,
}

async fn list_images(Extension(pool): Extension<sqlx::SqlitePool>) -> Json<Vec<ImageRecord>> {
    sqlx::query_as::<_, ImageRecord>("SELECT id, tags FROM images ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap()
        .into()
}

#[derive(Deserialize)]
struct Search {
    tags: String,
}

async fn search_images(
    Extension(pool): Extension<sqlx::SqlitePool>,
    Form(form): Form<Search>,
) -> Html<String> {
    let tag = format!("%{}%", form.tags);

    let rows = sqlx::query_as::<_, ImageRecord>(
        "SELECT id, tags FROM images WHERE tags LIKE ? ORDER BY id",
    )
    .bind(tag)
    .fetch_all(&pool)
    .await
    .unwrap();

    let mut results = String::new();
    for row in rows {
        results.push_str(&format!(
            "<a href=\"/image/{}\"><img src='/thumb/{}' /></a><br />",
            row.id, row.id
        ));
    }

    let path = std::path::Path::new("./search.html");
    let mut content = tokio::fs::read_to_string(path).await.unwrap();
    content = content.replace("{results}", &results);

    Html(content)
}
