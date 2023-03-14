use amqprs::{
    channel::{BasicPublishArguments, Channel},
    BasicProperties, DELIVERY_MODE_PERSISTENT,
};
use anyhow::Result;
use async_trait::async_trait;

use super::super::Publisher;

pub struct RabbitPublisher {
    pub(crate) channel: Channel,
    pub(crate) exchange: String,
    pub(crate) routing_key: String,
}

impl RabbitPublisher {
    pub fn new(channel: Channel, exchange: &str, routing_key: &str) -> Self {
        Self {
            channel,
            exchange: exchange.to_string(),
            routing_key: routing_key.to_string(),
        }
    }
}

#[async_trait]
impl Publisher for RabbitPublisher {
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
