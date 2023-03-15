use amqprs::channel::{
    BasicAckArguments, BasicConsumeArguments, BasicNackArguments, Channel, ConsumerMessage,
};
use anyhow::{Error, Result};
use async_trait::async_trait;
use std::pin::Pin;
use std::time::Duration;
use tokio_stream::{wrappers::UnboundedReceiverStream, Stream, StreamExt};

use super::super::ChunkReceiver;
use super::RabbitMessage;

#[allow(dead_code)]
pub struct RabbitChunkReceiver {
    chunk_stream: Pin<Box<dyn Stream<Item = Vec<ConsumerMessage>>>>,
    channel: Channel,
    pub consumer_tag: String,
    pub queue_name: String,
}

impl RabbitChunkReceiver {
    pub(crate) async fn new(
        channel: Channel,
        queue: &str,
        consumer_tag: &str,
        chunk_size: usize,
        duration: Duration,
    ) -> Result<Self> {
        let args = BasicConsumeArguments::new(queue, consumer_tag);
        let (_ctag, receiver) = channel.basic_consume_rx(args).await?;
        let chunk_receiver = UnboundedReceiverStream::new(receiver);
        let chunk_stream = chunk_receiver.chunks_timeout(chunk_size, duration);
        Ok(RabbitChunkReceiver {
            chunk_stream: Box::pin(chunk_stream),
            channel,
            consumer_tag: consumer_tag.to_string(),
            queue_name: queue.to_string(),
        })
    }
}

#[async_trait(?Send)]
impl ChunkReceiver for RabbitChunkReceiver {
    type Message = RabbitMessage;

    async fn receive(&mut self) -> Option<Vec<Self::Message>> {
        self.chunk_stream
            .next()
            .await
            .map(|vec| vec.into_iter().map(RabbitMessage).collect())
    }

    async fn ack(&self, message: &Self::Message, multiple: bool) -> Result<()> {
        self.channel
            .basic_ack(BasicAckArguments::new(
                message.0.deliver.as_ref().unwrap().delivery_tag(),
                multiple,
            ))
            .await
            .map_err(Error::from)
    }

    async fn nack(&self, message: &Self::Message, multiple: bool, requeue: bool) -> Result<()> {
        self.channel
            .basic_nack(BasicNackArguments::new(
                message.0.deliver.as_ref().unwrap().delivery_tag(),
                multiple,
                requeue,
            ))
            .await
            .map_err(Error::from)
    }
}
