use std::env;

use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{
        BasicAckArguments, BasicConsumeArguments, BasicPublishArguments, BasicQosArguments,
        Channel, ExchangeDeclareArguments, QueueBindArguments, QueueDeclareArguments,
    },
    connection::{Connection, OpenConnectionArguments},
    BasicProperties, DELIVERY_MODE_PERSISTENT,
};
use rust_rabbitmq::message_types::TestMessage;
use serde::Serialize;
use tokio::time;
use tracing::{info, metadata::LevelFilter, error};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Debug)]
enum ProcessResult {
    Success,
    Redeliver,
    Failure,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let processor_name = &args[1];

    set_up_logging();

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
    declare_topology(
        &channel,
        queue_name,
        routing_key,
        exchange_name,
        exchange_type,
    )
    .await;

    match processor_name.as_str() {
        "process" => process(&channel, queue_name).await,
        "generate" => test_generator(&channel, routing_key, exchange_name).await,
        _ => error!("unrecognised processor type"),
    }

    // explicitly close to gracefully shutdown
    channel.close().await.unwrap();
    connection.close().await.unwrap();
}

fn set_up_logging() {
    // Parse an `EnvFilter` configuration from the `RUST_LOG` environment variable.
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    // construct a subscriber that prints formatted traces to stdout
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .try_init()
        .ok();
}

async fn process(channel: &Channel, queue_name: &str) {
    info!("Starting process {queue_name}");

    let args = BasicConsumeArguments::new(queue_name, "edge_processor_tag");
    let (ctag, mut messages_rx) = channel.basic_consume_rx(args).await.unwrap();
    while let Some(message) = messages_rx.recv().await {
        let deliver = message.deliver.unwrap();
        let message: TestMessage = serde_json::from_slice(&message.content.unwrap()).unwrap();

        info!("{ctag} has received a message {:?}", message);
        let process_result = test_processor(message).await;
        info!("processor result {:?}", process_result);
        // need ability to batch process messages
        // the processor potentially need sql connection, blob storage connection string, redis connection, configurations, etc
        // there should be separation of the queuing logic and processing logic
        // once_cell global configuration

        // if doing batch processing, can set multple = true to ack multiple items up to the delivery tag
        channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
            .unwrap();

        time::sleep(time::Duration::from_millis(50)).await;
    }
}

async fn test_processor(message: TestMessage) -> ProcessResult {
    info!("processing message {:?}", message);
    ProcessResult::Success
}

async fn test_generator(channel: &Channel, routing_key: &str, exchange_name: &str) {
    for i in 0.. {
        let message = TestMessage {
            publisher: "example generator".to_string(),
            data: format!("hello world {i}"),
        };
        publish(channel, routing_key, exchange_name, message).await;
        time::sleep(time::Duration::from_millis(30)).await;
    }
}

async fn publish<T>(channel: &Channel, routing_key: &str, exchange_name: &str, message: T)
where
    T: Serialize + std::fmt::Display,
{
    // create arguments for basic_publish
    let args = BasicPublishArguments::new(exchange_name, routing_key);
    info!("sending message {message}");
    let content = serde_json::to_vec(&message).unwrap();
    channel
        .basic_publish(
            BasicProperties::default()
                .with_delivery_mode(DELIVERY_MODE_PERSISTENT)
                .finish(),
            content,
            args,
        )
        .await
        .unwrap();
}

async fn declare_topology(
    channel: &Channel,
    queue_name: &str,
    routing_key: &str,
    exchange_name: &str,
    exchange_type: &str,
) {
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

    // set limit to prefetch count
    // to make sure messages are evenly distributed among consumers
    // and prevent the consumer from being overwhelmed with messages
    // https://www.rabbitmq.com/confirms.html#channel-qos-prefetch-throughput
    channel
        .basic_qos(BasicQosArguments::new(0, 300, false))
        .await
        .unwrap();
}
