use std::collections::HashSet;
use std::env;

use dotenvy::dotenv;
use poise::serenity_prelude::GuildId;
use thiserror::Error;

pub struct AiConfig {
    pub model_access_key: String,
    pub default_model: String,
}

pub struct AppConfig {
    pub token: String,
    pub database_url: String,
    pub prefix: String,
    pub dev_guild_id: GuildId,
    pub allowed_guilds: HashSet<GuildId>,
    pub ai: AiConfig,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(".env file is required")]
    DotenvMissing,
    #[error("Failed to load .env: {0}")]
    DotenvLoad(dotenvy::Error),
    #[error("Missing {name} in environment.")]
    MissingEnv { name: &'static str },
    #[error("Invalid unicode value for {name} in environment.")]
    InvalidUnicode { name: &'static str },
    #[error("PREFIX cannot be empty")]
    EmptyPrefix,
    #[error("PREFIX must be a single prefix value without whitespace")]
    PrefixContainsWhitespace,
    #[error("DEV_GUILD_ID must be a valid u64 snowflake id")]
    InvalidDevGuildId(#[source] std::num::ParseIntError),
    #[error("ALLOWED_GUILD_IDS contains invalid id '{raw}'")]
    InvalidAllowedGuildId {
        raw: String,
        source: std::num::ParseIntError,
    },
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        load_dotenv()?;

        let token = required_env("DISCORD_TOKEN")?;
        let database_url = required_env("DATABASE_URL")?;
        let prefix = parse_prefix()?;
        let dev_guild_id = parse_dev_guild_id()?;
        let allowed_guilds = parse_allowed_guilds()?;
        let ai = parse_ai_config()?;

        Ok(Self {
            token,
            database_url,
            prefix,
            dev_guild_id,
            allowed_guilds,
            ai,
        })
    }
}

fn load_dotenv() -> Result<(), ConfigError> {
    match dotenv() {
        Ok(_) => Ok(()),
        Err(err) if err.not_found() => Err(ConfigError::DotenvMissing),
        Err(err) => Err(ConfigError::DotenvLoad(err)),
    }
}

fn required_env(name: &'static str) -> Result<String, ConfigError> {
    match env::var(name) {
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => Err(ConfigError::MissingEnv { name }),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::InvalidUnicode { name }),
    }
}

fn parse_prefix() -> Result<String, ConfigError> {
    let prefix = required_env("PREFIX")?;

    if prefix.trim().is_empty() {
        return Err(ConfigError::EmptyPrefix);
    }

    if prefix.chars().any(char::is_whitespace) {
        return Err(ConfigError::PrefixContainsWhitespace);
    }

    Ok(prefix)
}

fn parse_dev_guild_id() -> Result<GuildId, ConfigError> {
    let raw = required_env("DEV_GUILD_ID")?;
    let parsed = raw.parse::<u64>().map_err(ConfigError::InvalidDevGuildId)?;

    Ok(GuildId::new(parsed))
}

fn parse_allowed_guilds() -> Result<HashSet<GuildId>, ConfigError> {
    let raw = required_env("ALLOWED_GUILD_IDS")?;
    raw.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<u64>().map(GuildId::new).map_err(|source| {
                ConfigError::InvalidAllowedGuildId {
                    raw: s.to_string(),
                    source,
                }
            })
        })
        .collect()
}

fn parse_ai_config() -> Result<AiConfig, ConfigError> {
    let model_access_key = required_env("DO_MODEL_ACCESS_KEY")?;
    let default_model = env::var("AI_MODEL").unwrap_or_else(|_| "openai-gpt-oss-120b".to_string());

    Ok(AiConfig {
        model_access_key,
        default_model,
    })
}
