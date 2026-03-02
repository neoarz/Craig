use std::env;
use std::env::VarError;

use dotenvy::dotenv;
use poise::Prefix;
use tracing::warn;

use crate::Error;

pub struct AppConfig {
    pub token: String,
    pub database_url: String,
    pub primary_prefix: Option<String>,
    pub additional_prefixes: Vec<Prefix>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, Error> {
        load_dotenv()?;

        let token = env::var("DISCORD_TOKEN")
            .map_err(|_| "No discord token found in environment variables")?;
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "No database url found in environment variables")?;
        let (primary_prefix, additional_prefixes) = parse_prefixes()?;

        Ok(Self {
            token,
            database_url,
            primary_prefix,
            additional_prefixes,
        })
    }
}

fn load_dotenv() -> Result<(), Error> {
    match dotenv() {
        Ok(_) => Ok(()),
        Err(err) if err.not_found() => {
            if !dotenv_warning_disabled() {
                warn!(
                    "You have not included a .env file! If this is intentional you can disable this warning with `DISABLE_NO_DOTENV_WARNING=1`"
                );
            }
            Ok(())
        }
        Err(err) => Err(format!("Dotenv error: {err}").into()),
    }
}

fn dotenv_warning_disabled() -> bool {
    match env::var("DISABLE_NO_DOTENV_WARNING")
        .map(|x| x.to_ascii_lowercase())
        .as_deref()
    {
        Ok("1" | "true") => true,
        Ok("0" | "false") => false,
        Ok(_) => {
            panic!(
                "DISABLE_NO_DOTENV_WARNING environment variable is not a valid value (1/0/true/false)"
            )
        }
        Err(VarError::NotPresent) => false,
        Err(VarError::NotUnicode(err)) => panic!(
            "DISABLE_NO_DOTENV_WARNING environment variable is not set to valid Unicode, found: {:?}",
            err
        ),
    }
}

fn parse_prefixes() -> Result<(Option<String>, Vec<Prefix>), Error> {
    let unparsed = match env::var("PREFIXES") {
        Ok(unparsed) => unparsed,
        Err(VarError::NotPresent) => return Ok((None, Vec::new())),
        _ => return Err("Could not handle the environment variable for prefixes".into()),
    };

    let mut split = unparsed.split_whitespace().map(str::to_owned);

    let first = split
        .next()
        .ok_or("Could not parse prefixes from environment variables")?;

    let additional_prefixes = split
        .map(|prefix| Box::leak(prefix.into_boxed_str()) as &'static str)
        .map(Prefix::Literal)
        .collect();

    Ok((Some(first), additional_prefixes))
}
