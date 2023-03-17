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
        test_batch_processor::test_batch_process, test_db_processor::test_db_process,
        test_generator::test_generate, test_processor::test_process,
        test_protobuf_generator::test_protobuf_generate,
        test_protobuf_processor::test_protobuf_process,
        test_request_processor::test_request_process,
    },
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    dotenv().ok();

    let args = Cli::parse();
    let configs = Configs::new(&args.env)?;

    set_up_logging(args.is_local_run)?;

    let db = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(&configs.database.url)?;

    let rabbit_client = RabbitClient::new(&configs.rabbit).await?;

    info!("start processing in {}", args.env);
    match args.processor {
        Processors::TestProcess(args) => {
            test_process(rabbit_client.clone(), args.wait_ms, args.nack).await?
        }
        Processors::TestGenerate(args) => {
            test_generate(rabbit_client.clone(), args.wait_ms).await?
        }
        Processors::TestProtobufProcess => test_protobuf_process(rabbit_client.clone()).await?,
        Processors::TestProtobufGenerate => test_protobuf_generate(rabbit_client.clone()).await?,
        Processors::TestDBProcess(args) => test_db_process(db.clone(), args.wait_ms).await?,
        Processors::TestRequestProcess => test_request_process().await?,
        Processors::TestBatchProcess => test_batch_process(rabbit_client.clone()).await?,
    }

    // to make sure the life time of the rabbit connection outlives the channels
    // close the connection explicitly at the end of the program
    rabbit_client.close().await?;

    Ok(())
}
