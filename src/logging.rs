use tracing_subscriber::EnvFilter;

pub fn init() {
    let default_filter = format!("warn,{}=info", env!("CARGO_PKG_NAME"));
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}
