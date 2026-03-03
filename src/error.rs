use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Config(#[from] crate::config::ConfigError),
    #[error(transparent)]
    Serenity(Box<poise::serenity_prelude::Error>),
    #[error(transparent)]
    Sqlx(Box<sqlx::Error>),
    #[error(transparent)]
    SqlxMigration(Box<sqlx::migrate::MigrateError>),
    #[error(transparent)]
    Reqwest(Box<reqwest::Error>),
    #[error(transparent)]
    OpenAi(Box<async_openai::error::OpenAIError>),
    #[error("The AI model returned an empty response.")]
    EmptyAiResponse,
}

impl From<poise::serenity_prelude::Error> for AppError {
    fn from(value: poise::serenity_prelude::Error) -> Self {
        Self::Serenity(Box::new(value))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx(Box::new(value))
    }
}

impl From<sqlx::migrate::MigrateError> for AppError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        Self::SqlxMigration(Box::new(value))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(Box::new(value))
    }
}

impl From<async_openai::error::OpenAIError> for AppError {
    fn from(value: async_openai::error::OpenAIError) -> Self {
        Self::OpenAi(Box::new(value))
    }
}
