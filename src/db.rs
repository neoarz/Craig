use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::Error;

pub async fn connect_and_migrate(database_url: &str) -> Result<PgPool, Error> {
    let db = PgPoolOptions::new().connect(database_url).await?;
    sqlx::migrate!().run(&db).await?;

    Ok(db)
}
