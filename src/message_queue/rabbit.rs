use std::io::Cursor;

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
use serde::Deserialize;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::config::Rabbit;

use super::MessageQueuePublisher;
use super::MessageQueueReceiver;

static EXCHANGE: &str = "edge.direct";
static EXCHANGE_TYPE: &str = "direct";

#[derive(Clone)]
pub struct RabbitClient {
    // the rabbit connection is thread-safe so can be cloned across threads
    conn: Connection,
}

impl RabbitClient {
    pub async fn new(configs: &Rabbit) -> Result<Self> {
        let connection = Connection::open(&OpenConnectionArguments::new(
            &configs.host,
            configs.port,
            &configs.username,
            &configs.password,
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
        channel.close().await?;
        Ok(Self { conn: connection })
    }

    pub async fn close(self) -> Result<()> {
        self.conn.close().await?;
        Ok(())
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
        let args = QueueDeclareArguments::new(queue)
            .durable(true)
            // .arguments(FieldTable::new().insert("x-dead-letter-exchange", "some.exchange.name"))
            .finish();

        channel.queue_declare(args).await?.unwrap();

        // set routing_key the same as the queue name as we are using a direct exchange
        let routing_key = queue;
        // bind the queue to exchange
        channel
            .queue_bind(QueueBindArguments::new(queue, EXCHANGE, routing_key))
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
    async fn publish(&self, message_content: Vec<u8>) -> Result<()> {
        let args = BasicPublishArguments::new(&self.exchange, &self.routing_key);
        self.channel
            .basic_publish(
                BasicProperties::default()
                    .with_delivery_mode(DELIVERY_MODE_PERSISTENT)
                    .finish(),
                message_content,
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

    pub fn protobuf_deserialise<T>(&self) -> Result<T>
    where
        T: prost::Message + std::default::Default,
    {
        let message_data = T::decode(&mut Cursor::new(self.0.content.as_ref().unwrap()))?;
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
