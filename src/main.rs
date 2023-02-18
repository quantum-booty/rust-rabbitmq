use anyhow::Result;
use clap::Parser;
use tracing::info;

use rust_rabbitmq::{
    cli::{Cli, Processors},
    log::set_up_logging,
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
