use tracing::metadata::LevelFilter;
use tracing_subscriber::EnvFilter;

pub fn init() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::WARN.into())
                .from_env_lossy(),
        )
        .init();
}
