use std::env;

use dotenvy::dotenv;
use poise::serenity_prelude::GuildId;

use crate::Error;

pub struct AiConfig {
    pub model_access_key: String,
    pub default_model: String,
}

pub struct AppConfig {
    pub token: String,
    pub database_url: String,
    pub prefix: String,
    pub dev_guild_id: GuildId,
    pub ai: AiConfig,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, Error> {
        load_dotenv()?;

        let token =
            env::var("DISCORD_TOKEN").map_err(|_| "Missing DISCORD_TOKEN in environment.")?;
        let database_url =
            env::var("DATABASE_URL").map_err(|_| "Missing DATABASE_URL in environment.")?;
        let prefix = parse_prefix()?;
        let dev_guild_id = parse_dev_guild_id()?;
        let ai = parse_ai_config()?;

        Ok(Self {
            token,
            database_url,
            prefix,
            dev_guild_id,
            ai,
        })
    }
}

fn load_dotenv() -> Result<(), Error> {
    match dotenv() {
        Ok(_) => Ok(()),
        Err(err) if err.not_found() => Err(".env file is required".into()),
        Err(err) => Err(format!("Failed to load .env: {err}").into()),
    }
}

fn parse_prefix() -> Result<String, Error> {
    let prefix = env::var("PREFIX").map_err(|_| "Missing PREFIX in environment.")?;

    if prefix.trim().is_empty() {
        return Err("PREFIX cannot be empty".into());
    }

    if prefix.chars().any(char::is_whitespace) {
        return Err("PREFIX must be a single prefix value without whitespace".into());
    }

    Ok(prefix)
}

fn parse_dev_guild_id() -> Result<GuildId, Error> {
    let raw = env::var("DEV_GUILD_ID").map_err(|_| "Missing DEV_GUILD_ID in environment.")?;

    let parsed = raw
        .parse::<u64>()
        .map_err(|_| "DEV_GUILD_ID must be a valid u64 snowflake id")?;

    Ok(GuildId::new(parsed))
}

fn parse_ai_config() -> Result<AiConfig, Error> {
    let model_access_key = env::var("DO_MODEL_ACCESS_KEY")
        .map_err(|_| "Missing DO_MODEL_ACCESS_KEY in environment.")?;
    let default_model = env::var("AI_MODEL").unwrap_or_else(|_| "openai-gpt-oss-120b".to_string());

    Ok(AiConfig {
        model_access_key,
        default_model,
    })
}
