use tracing::metadata::LevelFilter;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn set_up_logging() {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    // Alternatively can parse an `EnvFilter` configuration from the `RUST_LOG` environment variable
    // e.g. by setting an environment variable RUST_LOG=debug
    // EnvFilter::from_default_env()

    // construct a subscriber that prints formatted traces to stdout
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .try_init()
        .ok();
}
