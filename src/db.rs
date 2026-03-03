use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::Error;

pub struct ChatLog {
    pub user_id: i64,
    pub username: String,
    pub guild_id: Option<i64>,
    pub channel_id: i64,
    pub source: &'static str,
    pub model: String,
    pub prompt: String,
    pub response: String,
    pub latency_ms: i32,
    pub success: bool,
    pub error_text: Option<String>,
}

pub async fn connect_and_migrate(database_url: &str) -> Result<PgPool, Error> {
    let db = PgPoolOptions::new().connect(database_url).await?;
    sqlx::migrate!().run(&db).await?;

    Ok(db)
}

pub async fn insert_chat_log(db: &PgPool, log: &ChatLog) {
    if let Err(error) = sqlx::query(
        "INSERT INTO chat_logs
            (user_id, username, guild_id, channel_id, source, model, prompt, response, latency_ms, success, error_text)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(log.user_id)
    .bind(&log.username)
    .bind(log.guild_id)
    .bind(log.channel_id)
    .bind(log.source)
    .bind(&log.model)
    .bind(&log.prompt)
    .bind(&log.response)
    .bind(log.latency_ms)
    .bind(log.success)
    .bind(&log.error_text)
    .execute(db)
    .await
    {
        crate::app_error!("Failed to log chat interaction: {error}");
    }
}
