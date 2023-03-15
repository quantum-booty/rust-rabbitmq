use amqprs::channel::{BasicAckArguments, BasicConsumeArguments, Channel, ConsumerMessage};
use anyhow::{Error, Result};
use async_trait::async_trait;
use tokio::sync::mpsc::UnboundedReceiver;

use super::super::Receiver;
use super::RabbitMessage;

#[allow(dead_code)]
pub struct RabbitReceiver {
    receiver: UnboundedReceiver<ConsumerMessage>,
    channel: Channel,
    pub consumer_tag: String,
    pub queue_name: String,
}

impl RabbitReceiver {
    pub(crate) async fn new(channel: Channel, queue: &str, consumer_tag: &str) -> Result<Self> {
        let args = BasicConsumeArguments::new(queue, consumer_tag);
        let (_ctag, messages_rx) = channel.basic_consume_rx(args).await?;
        Ok(RabbitReceiver {
            receiver: messages_rx,
            channel,
            consumer_tag: consumer_tag.to_string(),
            queue_name: queue.to_string(),
        })
    }
}

#[async_trait]
impl Receiver for RabbitReceiver {
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
