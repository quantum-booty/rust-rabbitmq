use async_trait::async_trait;
use anyhow::Result;
use serde::Serialize;

pub mod rabbit;

#[async_trait]
pub trait MessageQueueReceiver {
    type Message;
    async fn receive(&mut self) -> Result<Self::Message>;
}

#[async_trait]
pub trait MessageQueuePublisher {
    async fn publish<T>(&self, message: T) -> Result<()>
    where
        T: Serialize + std::fmt::Display + Send;
}
