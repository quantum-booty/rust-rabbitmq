use anyhow::Result;
use clap::Parser;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing::info;

use rust_rabbitmq::{
    cli::{Cli, Processors},
    config::Configs,
    log::set_up_logging,
    message_queue::rabbit::RabbitClient,
    processors::{
        test_db_processor::test_db_process, test_generator::test_generate,
        test_processor::test_process, test_request_processor::test_request_process,
    },
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let args = Cli::parse();
    dotenv().expect(".env file not found");
    let configs = Configs::new(&args.env)?;

    set_up_logging()?;

    let db = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(&configs.database.url)?;

    let rabbit_client = RabbitClient::new(&configs.rabbit).await?;

    info!("start processing in {}", args.env);
    match args.processor {
        Processors::TestProcess(args) => test_process(rabbit_client, args.wait_ms).await?,
        Processors::TestGenerate(args) => test_generate(rabbit_client, args.wait_ms).await?,
        Processors::TestDBProcess(args) => test_db_process(db.clone(), args.wait_ms).await?,
        Processors::TestRequestProcess => test_request_process().await?,
    }

    Ok(())
}
