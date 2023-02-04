use std::env;

use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{
        BasicAckArguments, BasicConsumeArguments, BasicPublishArguments, Channel,
        ExchangeDeclareArguments, QueueBindArguments, QueueDeclareArguments,
    },
    connection::{Connection, OpenConnectionArguments},
    BasicProperties,
};
use serde::{Deserialize, Serialize};
use tokio::time;
use tracing::{info, metadata::LevelFilter};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Deserialize, Serialize, Debug)]
struct TestMessage {
    publisher: String,
    data: String,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let run_type = &args[1];

    // construct a subscriber that prints formatted traces to stdout
    tracing_subscriber::registry()
        .with(fmt::layer())
        // .with(EnvFilter::from_default_env())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .try_init()
        .ok();

    // open a connection to RabbitMQ server
    let connection = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "guest",
        "guest",
    ))
    .await
    .unwrap();

    connection
        .register_callback(DefaultConnectionCallback)
        .await
        .unwrap();

    // open a channel on the connection
    let channel = connection.open_channel(None).await.unwrap();
    channel
        .register_callback(DefaultChannelCallback)
        .await
        .unwrap();

    let queue_name = "edge.do_something_processor";
    let routing_key = "edge.do_something_processor";
    let exchange_name = "edge.direct";
    let exchange_type = "direct";
    let queue_name = declare_topology(
        &channel,
        queue_name,
        routing_key,
        exchange_name,
        exchange_type,
    )
    .await;

    match run_type.as_str() {
        "process" => process(&channel, &queue_name).await,
        "generate" => generate(&channel, routing_key, exchange_name).await,
        _ => panic!("unrecognised run type"),
    }

    // explicitly close to gracefully shutdown
    channel.close().await.unwrap();
    connection.close().await.unwrap();
}

async fn process(channel: &Channel, queue_name: &str) {
    info!("Starting process {queue_name}");

    let args = BasicConsumeArguments::new(queue_name, "edge_processor_tag").finish();
    let (ctag, mut messages_rx) = channel.basic_consume_rx(args).await.unwrap();
    while let Some(message) = messages_rx.recv().await {
        let deliver = message.deliver.unwrap();
        let message: TestMessage = serde_json::from_slice(&message.content.unwrap()).unwrap();

        info!("{ctag} has received a message {:?}", message);

        channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
            .unwrap();

        time::sleep(time::Duration::from_millis(50)).await;
    }
}

async fn generate(channel: &Channel, routing_key: &str, exchange_name: &str) {
    // create arguments for basic_publish
    let args = BasicPublishArguments::new(exchange_name, routing_key);
    for i in 0.. {
        let message = TestMessage {
            publisher: "example generator".to_string(),
            data: format!("hello world {i}"),
        };
        info!("sending message {message:?}");
        let content = serde_json::to_vec(&message).unwrap();
        channel
            .basic_publish(BasicProperties::default(), content, args.clone())
            .await
            .unwrap();
        time::sleep(time::Duration::from_millis(30)).await;
    }
}

async fn declare_topology(
    channel: &Channel,
    queue_name: &str,
    routing_key: &str,
    exchange_name: &str,
    exchange_type: &str,
) -> String {
    // declare exchange
    channel
        .exchange_declare(
            ExchangeDeclareArguments::new(exchange_name, exchange_type)
                .durable(true)
                .finish(),
        )
        .await
        .unwrap();

    // declare a queue
    let (queue_name, _, _) = channel
        .queue_declare(
            QueueDeclareArguments::new(queue_name)
                .durable(true)
                .finish(),
        )
        .await
        .unwrap()
        .unwrap();

    // bind the queue to exchange
    channel
        .queue_bind(QueueBindArguments::new(
            &queue_name,
            exchange_name,
            routing_key,
        ))
        .await
        .unwrap();

    queue_name
}
