use std::env;
use std::env::VarError;

use dotenvy::dotenv;
use poise::serenity_prelude::GuildId;

use crate::Error;

pub struct AppConfig {
    pub token: String,
    pub database_url: String,
    pub prefix: String,
    pub dev_guild_id: Option<GuildId>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, Error> {
        load_dotenv()?;

        let token = env::var("DISCORD_TOKEN")
            .map_err(|_| "No discord token found in environment variables")?;
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "No database url found in environment variables")?;
        let prefix = parse_prefix()?;
        let dev_guild_id = parse_dev_guild_id()?;

        Ok(Self {
            token,
            database_url,
            prefix,
            dev_guild_id,
        })
    }
}

fn load_dotenv() -> Result<(), Error> {
    match dotenv() {
        Ok(_) => Ok(()),
        Err(err) if err.not_found() => Err(".env file is required".into()),
        Err(err) => Err(format!("Dotenv error: {err}").into()),
    }
}

fn parse_prefix() -> Result<String, Error> {
    let prefix = env::var("PREFIX").map_err(|_| "No PREFIX found in environment variables")?;

    if prefix.trim().is_empty() {
        return Err("PREFIX cannot be empty".into());
    }

    if prefix.chars().any(char::is_whitespace) {
        return Err("PREFIX must be a single prefix value without whitespace".into());
    }

    Ok(prefix)
}

fn parse_dev_guild_id() -> Result<Option<GuildId>, Error> {
    let raw = match env::var("DEV_GUILD_ID") {
        Ok(raw) => raw,
        Err(VarError::NotPresent) => return Ok(None),
        Err(_) => return Err("Could not handle the environment variable for dev guild id".into()),
    };

    let parsed = raw
        .parse::<u64>()
        .map_err(|_| "DEV_GUILD_ID must be a valid u64 snowflake id")?;

    Ok(Some(GuildId::new(parsed)))
}
