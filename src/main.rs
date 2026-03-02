use sqlx::PgPool;

mod bot;
mod commands;
mod config;
mod db;
mod logging;

pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct Data {
    pub db: PgPool,
}

#[tokio::main]
async fn main() {
    logging::init();

    if let Err(error) = run().await {
        tracing::error!("Fatal error: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Error> {
    let config = config::AppConfig::from_env()?;
    let db = db::connect_and_migrate(&config.database_url).await?;
    let data = Data { db: db.clone() };
    bot::start(config, data, db).await
}
