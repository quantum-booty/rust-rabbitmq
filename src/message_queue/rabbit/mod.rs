mod chunk_receiver;
mod publisher;
mod receiver;

use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{
        BasicQosArguments, Channel, ConsumerMessage, ExchangeDeclareArguments, QueueBindArguments,
        QueueDeclareArguments,
    },
    connection::{Connection, OpenConnectionArguments},
};
use anyhow::Result;
use serde::Deserialize;
use std::{io::Cursor, time::Duration};

use crate::config::Rabbit;

use self::{
    chunk_receiver::RabbitChunkReceiver, publisher::RabbitPublisher, receiver::RabbitReceiver,
};

use super::Publisher;

static EXCHANGE: &str = "edge.direct";
static EXCHANGE_TYPE: &str = "direct";

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

    pub async fn get_publisher(&self, queue: &str) -> Result<impl Publisher> {
        let channel = Self::get_channel(&self.conn).await?;
        self.declare_queue(&channel, queue).await?;
        Ok(RabbitPublisher::new(channel, EXCHANGE, queue))
    }

    pub async fn get_receiver(
        &self,
        queue: &str,
        tag: &str,
        prefetch_count: u16,
    ) -> Result<RabbitReceiver> {
        let channel = Self::get_channel(&self.conn).await?;
        self.declare_queue(&channel, queue).await?;
        // set limit to prefetch count
        // to make sure messages are evenly distributed among consumers
        // and prevent the consumer from being overwhelmed with messages
        // https://www.rabbitmq.com/confirms.html#channel-qos-prefetch-throughput
        channel
            .basic_qos(BasicQosArguments::new(0, prefetch_count, false))
            .await?;
        RabbitReceiver::new(channel, queue, tag).await
    }

    pub async fn get_chunk_receiver(
        &self,
        queue: &str,
        tag: &str,
        prefetch_count: u16,
        chunk_size: usize,
        duration: Duration,
    ) -> Result<RabbitChunkReceiver> {
        let channel = Self::get_channel(&self.conn).await?;
        self.declare_queue(&channel, queue).await?;
        // set limit to prefetch count
        // to make sure messages are evenly distributed among consumers
        // and prevent the consumer from being overwhelmed with messages
        // https://www.rabbitmq.com/confirms.html#channel-qos-prefetch-throughput
        channel
            .basic_qos(BasicQosArguments::new(0, prefetch_count, false))
            .await?;

        RabbitChunkReceiver::new(channel, queue, tag, chunk_size, duration).await
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
