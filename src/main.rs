use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{Channel, ExchangeDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
};
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tracing::{error, info, metadata::LevelFilter};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use rust_rabbitmq::{
    message_queue::rabbit::{EXCHANGE, EXCHANGE_TYPE},
    processors::{test_generator::test_generate, test_processor::test_process},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // TODO: use clap for cli
    use std::env;
    let args: Vec<String> = env::args().collect();
    let processor_name = &args[1];

    set_up_logging();

    let connection = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "guest",
        "guest",
    ))
    .await?;

    connection
        .register_callback(DefaultConnectionCallback)
        .await?;

    // open a channel on the connection
    let channel = connection.open_channel(None).await?;
    channel.register_callback(DefaultChannelCallback).await?;

    // declare exchange
    channel
        .exchange_declare(
            ExchangeDeclareArguments::new(EXCHANGE, EXCHANGE_TYPE)
                .durable(true)
                .finish(),
        )
        .await?;

    info!("start processing");
    if let Err(err) = process(Arc::new(channel), processor_name).await {
        error!("{err:?}");
    }

    // explicitly close to gracefully shutdown
    // channel.close().await?;
    connection.close().await?;

    Ok(())
}

async fn process(channel: Arc<Channel>, processor_name: &str) -> Result<()> {
    match processor_name {
        "process" => test_process(channel).await,
        "generate" => test_generate(channel).await,
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
