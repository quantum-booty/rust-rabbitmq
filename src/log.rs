use anyhow::Result;
use tracing_subscriber::EnvFilter;

pub fn set_up_logging(is_local_run: bool) -> Result<()> {
    if is_local_run {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    } else {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    }
    Ok(())
}
