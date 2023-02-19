use anyhow::Result;

pub fn set_up_logging() -> Result<()> {
    tracing_subscriber::fmt::init();
    Ok(())
}
