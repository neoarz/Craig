use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::Error;

pub async fn connect_and_migrate(database_url: &str) -> Result<PgPool, Error> {
    tracing::info!("Connecting to Postgres...");
    let db = PgPoolOptions::new().connect(database_url).await?;
    tracing::info!("Connected to Postgres");

    tracing::info!("Applying migrations...");
    sqlx::migrate!().run(&db).await?;
    tracing::info!("Migrations applied");

    Ok(db)
}
