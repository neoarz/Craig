use tracing_subscriber::EnvFilter;

pub(crate) const BOT_TARGET: &str = "Craig::bot";
pub(crate) const RUN_TARGET: &str = "Craig::run";
pub(crate) const ERROR_TARGET: &str = "Craig::error";

#[macro_export]
macro_rules! bot_info {
    ($($arg:tt)*) => {
        tracing::info!(target: $crate::logging::BOT_TARGET, $($arg)*)
    };
}

#[macro_export]
macro_rules! bot_warn {
    ($($arg:tt)*) => {
        tracing::warn!(target: $crate::logging::BOT_TARGET, $($arg)*)
    };
}

#[macro_export]
macro_rules! run_debug {
    ($($arg:tt)*) => {
        tracing::debug!(target: $crate::logging::RUN_TARGET, $($arg)*)
    };
}

#[macro_export]
macro_rules! app_error {
    ($($arg:tt)*) => {
        tracing::error!(target: $crate::logging::ERROR_TARGET, $($arg)*)
    };
}

pub fn init() {
    let default_filter = format!("warn,{}=info,Craig::run=debug", env!("CARGO_PKG_NAME"));
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}
