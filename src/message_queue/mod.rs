use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;

pub mod rabbit;

#[async_trait]
pub trait MessageQueueReceiver {
    type Message;
    type AckId;
    async fn receive(&mut self) -> Option<(Self::Message, Self::AckId)>;
    async fn ack(&self, ack_id: Self::AckId) -> Result<()>;
}

#[async_trait]
pub trait MessageQueuePublisher {
    async fn publish<T>(&self, message: T) -> Result<()>
    where
        T: Serialize + std::fmt::Display + Send;
}
