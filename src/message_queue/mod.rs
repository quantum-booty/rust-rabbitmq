use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;

pub mod rabbit;

#[async_trait]
pub trait MessageQueueReceiver {
    type Message;
    async fn receive(&mut self) -> Option<Self::Message>;
    async fn ack(&self, message: &Self::Message) -> Result<()>;
}

#[async_trait]
pub trait MessageQueuePublisher {
    async fn publish<T>(&self, message: T) -> Result<()>
    where
        T: Serialize + std::fmt::Display + Send;
}
