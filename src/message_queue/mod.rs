use anyhow::Result;
use async_trait::async_trait;

pub mod rabbit;

#[async_trait]
pub trait Receiver {
    type Message;
    async fn receive(&mut self) -> Option<Self::Message>;
    async fn ack(&self, message: &Self::Message) -> Result<()>;
}

#[async_trait(?Send)]
pub trait ChunkReceiver {
    type Message;
    async fn receive(&mut self) -> Option<Vec<Self::Message>>;
    async fn ack_many(&self, messages: &[Self::Message]) -> Result<()>;
    async fn ack(&self, message: &Self::Message) -> Result<()>;
}

#[async_trait]
pub trait Publisher {
    async fn publish(&self, message: Vec<u8>) -> Result<()>;
}
