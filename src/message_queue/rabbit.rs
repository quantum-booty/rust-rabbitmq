use std::sync::Arc;

use amqprs::{
    channel::{
        BasicConsumeArguments, BasicPublishArguments, BasicQosArguments, Channel, ConsumerMessage,
        QueueBindArguments, QueueDeclareArguments,
    },
    BasicProperties, DELIVERY_MODE_PERSISTENT,
};
use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::info;

use super::MessageQueuePublisher;
use super::MessageQueueReceiver;

pub static EXCHANGE: &str = "edge.direct";
pub static EXCHANGE_TYPE: &str = "direct";

pub struct RabbitQueueMessagePublisher {
    channel: Arc<Channel>,
    exchange_name: String,
    routing_key: String,
}

impl RabbitQueueMessagePublisher {
    pub fn new(channel: Arc<Channel>, exchange_name: &str, routing_key: &str) -> Self {
        Self {
            channel,
            exchange_name: exchange_name.to_string(),
            routing_key: routing_key.to_string(),
        }
    }
}

#[async_trait]
impl MessageQueuePublisher for RabbitQueueMessagePublisher {
    async fn publish<T>(&self, message: T) -> Result<()>
    where
        T: Serialize + std::fmt::Display + Send,
    {
        let args = BasicPublishArguments::new(&self.exchange_name, &self.routing_key);
        info!("sending message {message}");
        // todo: The serialization method can be made abstract
        let content = serde_json::to_vec(&message)?;
        self.channel
            .basic_publish(
                BasicProperties::default()
                    .with_delivery_mode(DELIVERY_MODE_PERSISTENT)
                    .with_content_type("application/json")
                    .finish(),
                content,
                args,
            )
            .await?;
        Ok(())
    }
}

pub struct RabbitMessageQueueReceiver {
    receiver: UnboundedReceiver<ConsumerMessage>,
    consumer_tag: String,
    queue_name: String,
}

impl RabbitMessageQueueReceiver {
    pub async fn new(channel: &Channel, queue_name: &str, consumer_tag: &str) -> Result<Self> {
        let args = BasicConsumeArguments::new(queue_name, consumer_tag);
        let (_ctag, messages_rx) = channel.basic_consume_rx(args).await?;
        Ok(RabbitMessageQueueReceiver {
            receiver: messages_rx,
            consumer_tag: consumer_tag.to_string(),
            queue_name: queue_name.to_string(),
        })
    }
}

#[async_trait]
impl MessageQueueReceiver for RabbitMessageQueueReceiver {
    type Message = Option<ConsumerMessage>;
    async fn receive(&mut self) -> Result<Self::Message> {
        Ok(self.receiver.recv().await)
    }
}

pub async fn declare_queue(
    channel: &Channel,
    exchange: &str,
    queue_name: &str,
    prefetch_count: u16,
) -> Result<()> {
    // declare a queue
    let (queue_name, _, _) = channel
        .queue_declare(
            QueueDeclareArguments::new(queue_name)
                .durable(true)
                .finish(),
        )
        .await?
        .unwrap();

    // bind the queue to exchange
    channel
        .queue_bind(QueueBindArguments::new(&queue_name, exchange, &queue_name))
        .await?;

    // set limit to prefetch count
    // to make sure messages are evenly distributed among consumers
    // and prevent the consumer from being overwhelmed with messages
    // https://www.rabbitmq.com/confirms.html#channel-qos-prefetch-throughput
    channel
        .basic_qos(BasicQosArguments::new(0, prefetch_count, false))
        .await?;

    Ok(())
}
