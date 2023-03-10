use anyhow::Result;
use async_trait::async_trait;

pub mod rabbit;

#[async_trait]
pub trait MessageQueueReceiver {
    type Message;
    async fn receive(&mut self) -> Option<Self::Message>;
    async fn ack(&self, message: &Self::Message) -> Result<()>;
}

#[async_trait]
pub trait MessageQueuePublisher {
    async fn publish(&self, message: Vec<u8>) -> Result<()>;
}
