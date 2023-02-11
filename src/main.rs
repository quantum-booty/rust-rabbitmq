use anyhow::{anyhow, Result};
use tracing::{error, info, metadata::LevelFilter};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use rust_rabbitmq::{
    message_queue::rabbit::RabbitClient,
    processors::{test_generator::test_generate, test_processor::test_process},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // TODO: use clap for cli
    use std::env;
    let args: Vec<String> = env::args().collect();
    let processor_name = &args[1];

    set_up_logging();

    let rabbit_client = RabbitClient::new().await?;

    info!("start processing");
    if let Err(err) = process(rabbit_client, processor_name).await {
        error!("{err:?}");
    }
    Ok(())
}

async fn process(rabbit_client: RabbitClient, processor_name: &str) -> Result<()> {
    match processor_name {
        "process" => test_process(rabbit_client).await,
        "generate" => test_generate(rabbit_client).await,
        _ => Err(anyhow!("unrecognised processor type")),
    }
}

fn set_up_logging() {
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
