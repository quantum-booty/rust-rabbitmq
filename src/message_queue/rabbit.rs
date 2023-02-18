use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{
        BasicAckArguments, BasicConsumeArguments, BasicPublishArguments, BasicQosArguments,
        Channel, ConsumerMessage, ExchangeDeclareArguments, QueueBindArguments,
        QueueDeclareArguments,
    },
    connection::{Connection, OpenConnectionArguments},
    BasicProperties, DELIVERY_MODE_PERSISTENT,
};
use anyhow::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;

use super::MessageQueuePublisher;
use super::MessageQueueReceiver;

static EXCHANGE: &str = "edge.direct";
static EXCHANGE_TYPE: &str = "direct";

pub struct RabbitClient {
    conn: Connection,
}

impl RabbitClient {
    pub async fn new() -> Result<Self> {
        // TODO: don't hard code
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
        let channel = Self::get_channel(&connection).await?;
        channel
            .exchange_declare(
                ExchangeDeclareArguments::new(EXCHANGE, EXCHANGE_TYPE)
                    .durable(true)
                    .finish(),
            )
            .await?;
        Ok(Self { conn: connection })
    }
    pub async fn get_publisher(&self, queue: &str) -> Result<impl MessageQueuePublisher> {
        let channel = Self::get_channel(&self.conn).await?;
        self.declare_queue(&channel, queue).await?;
        Ok(RabbitQueueMessagePublisher::new(channel, EXCHANGE, queue))
    }

    pub async fn get_receiver(
        &self,
        queue: &str,
        tag: &str,
        prefetch_count: u16,
    ) -> Result<RabbitMessageQueueReceiver> {
        let channel = Self::get_channel(&self.conn).await?;
        self.declare_queue(&channel, queue).await?;
        // set limit to prefetch count
        // to make sure messages are evenly distributed among consumers
        // and prevent the consumer from being overwhelmed with messages
        // https://www.rabbitmq.com/confirms.html#channel-qos-prefetch-throughput
        channel
            .basic_qos(BasicQosArguments::new(0, prefetch_count, false))
            .await?;
        RabbitMessageQueueReceiver::new(channel, queue, tag).await
    }

    async fn declare_queue(&self, channel: &Channel, queue: &str) -> Result<()> {
        channel
            .queue_declare(QueueDeclareArguments::new(queue).durable(true).finish())
            .await?
            .unwrap();

        // bind the queue to exchange
        channel
            .queue_bind(QueueBindArguments::new(queue, EXCHANGE, queue))
            .await?;
        Ok(())
    }

    async fn get_channel(conn: &Connection) -> Result<Channel> {
        let channel = conn.open_channel(None).await?;
        channel.register_callback(DefaultChannelCallback).await?;
        Ok(channel)
    }
}

pub struct RabbitQueueMessagePublisher {
    channel: Channel,
    exchange: String,
    routing_key: String,
}

impl RabbitQueueMessagePublisher {
    pub fn new(channel: Channel, exchange: &str, routing_key: &str) -> Self {
        Self {
            channel,
            exchange: exchange.to_string(),
            routing_key: routing_key.to_string(),
        }
    }
}

#[async_trait]
impl MessageQueuePublisher for RabbitQueueMessagePublisher {
    async fn publish<T>(&self, message: T) -> Result<()>
    where
        T: Serialize + Send,
    {
        let args = BasicPublishArguments::new(&self.exchange, &self.routing_key);
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

#[allow(dead_code)]
pub struct RabbitMessageQueueReceiver {
    receiver: UnboundedReceiver<ConsumerMessage>,
    channel: Channel,
    consumer_tag: String,
    queue_name: String,
}

impl RabbitMessageQueueReceiver {
    pub async fn new(channel: Channel, queue: &str, consumer_tag: &str) -> Result<Self> {
        let args = BasicConsumeArguments::new(queue, consumer_tag);
        let (_ctag, messages_rx) = channel.basic_consume_rx(args).await?;
        Ok(RabbitMessageQueueReceiver {
            receiver: messages_rx,
            channel,
            consumer_tag: consumer_tag.to_string(),
            queue_name: queue.to_string(),
        })
    }
}

pub struct RabbitMessage(ConsumerMessage);

impl RabbitMessage {
    pub fn json_deserialise<T>(&self) -> Result<T>
    where
        for<'a> T: Deserialize<'a>,
    {
        let message_data: T = serde_json::from_slice(self.0.content.as_ref().unwrap())?;
        Ok(message_data)
    }
}

#[async_trait]
impl MessageQueueReceiver for RabbitMessageQueueReceiver {
    type Message = RabbitMessage;
    async fn receive(&mut self) -> Option<Self::Message> {
        self.receiver.recv().await.map(RabbitMessage)
    }

    async fn ack(&self, message: &Self::Message) -> Result<()> {
        self.channel
            .basic_ack(BasicAckArguments::new(
                message.0.deliver.as_ref().unwrap().delivery_tag(),
                false,
            ))
            .await
            .map_err(Error::from)
    }
}
