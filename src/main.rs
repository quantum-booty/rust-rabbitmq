use anyhow::Result;
use clap::Parser;
use tracing::{info, metadata::LevelFilter};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use rust_rabbitmq::{
    cli::{Cli, Processors},
    message_queue::rabbit::RabbitClient,
    processors::{test_generator::test_generate, test_processor::test_process},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let args = Cli::parse();

    set_up_logging();

    let rabbit_client = RabbitClient::new().await?;

    info!("start processing in {}", args.env);
    match args.processor {
        Processors::TestProcess(args) => test_process(rabbit_client, args.wait_ms).await?,
        Processors::TestGenerate(args) => test_generate(rabbit_client, args.wait_ms).await?,
    }

    Ok(())
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
