use futures::TryStreamExt;
use sqlx::{FromRow, Row};

#[derive(Debug, FromRow)]
struct Message {
    id: i64,
    message: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    let db_url = std::env::var("DATABASE_URL")?;

    let pool = sqlx::SqlitePool::connect(&db_url).await?;

    // run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    query_database_manual_mapping(&pool).await?;
    query_database_auto_mapping(&pool).await?;
    query_database_with_stream(&pool).await?;

    update_message(4, "updated message", &pool).await?;

    query_database_with_stream(&pool).await?;
    Ok(())
}

async fn query_database_manual_mapping(pool: &sqlx::SqlitePool) -> anyhow::Result<()> {
    let messages = sqlx::query("SELECT id, message FROM messages")
        .map(|row: sqlx::sqlite::SqliteRow| {
            let id: i64 = row.get(0);
            let message: String = row.get(1);
            (id, message)
        })
        .fetch_all(pool)
        .await?;

    for (id, message) in messages {
        println!("{id}: {message}");
    }
    Ok(())
}

async fn query_database_auto_mapping(pool: &sqlx::SqlitePool) -> anyhow::Result<()> {
    let messages = sqlx::query_as::<_, Message>("SELECT id, message FROM messages")
        .fetch_all(pool)
        .await?;

    for message in messages {
        println!("{}: {}", message.id, message.message);
    }
    Ok(())
}

async fn query_database_with_stream(pool: &sqlx::SqlitePool) -> anyhow::Result<()> {
    let mut message_stream =
        sqlx::query_as::<_, Message>("SELECT id, message FROM messages").fetch(pool);

    while let Some(message) = message_stream.try_next().await? {
        println!("{message:?}");
    }
    Ok(())
}

async fn update_message(id: i64, message: &str, pool: &sqlx::SqlitePool) -> anyhow::Result<()> {
    sqlx::query("UPDATE messages SET message = ? WHERE id =?")
        .bind(message)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
