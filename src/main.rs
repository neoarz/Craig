use poise::serenity_prelude::GuildId;
use sqlx::PgPool;

mod bot;
mod commands;
mod config;
mod db;
mod helpers;
mod logging;
mod services;

pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct Data {
    pub db: PgPool,
    pub dev_guild_id: GuildId,
    pub ai: services::ai::AiService,
}

#[tokio::main]
async fn main() {
    logging::init();

    if let Err(error) = run().await {
        crate::app_error!("{error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Error> {
    let config = config::AppConfig::from_env()?;
    let db = db::connect_and_migrate(&config.database_url).await?;
    let ai = services::ai::AiService::new(&config.ai)?;
    let data = Data {
        db: db.clone(),
        dev_guild_id: config.dev_guild_id,
        ai,
    };
    bot::start(config, data, db).await
}
