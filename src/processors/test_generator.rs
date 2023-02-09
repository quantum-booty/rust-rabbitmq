use amqprs::channel::Channel;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::time;

use crate::{
    message_queue::{
        rabbit::{declare_queue, RabbitQueueMessagePublisher, EXCHANGE},
        MessageQueuePublisher,
    },
    message_types::TestMessage,
};

use super::Processor;

pub struct TestGenerator {
    channel: Arc<Channel>,
    publisher: RabbitQueueMessagePublisher,
}

impl TestGenerator {
    pub async fn new(channel: Arc<Channel>) -> Result<Self> {
        let queue_name = "edge.do_something_processor";
        declare_queue(&channel, EXCHANGE, queue_name, 1).await?;
        let publisher = RabbitQueueMessagePublisher::new(channel.clone(), EXCHANGE, queue_name);
        Ok(Self { channel, publisher })
    }
}

#[async_trait]
impl Processor for TestGenerator {
    async fn run(&mut self) -> Result<()> {
        for i in 0.. {
            let message = TestMessage {
                publisher: "example generator".to_string(),
                data: format!("hello world {i}"),
            };

            self.publisher.publish(message).await?;
            time::sleep(time::Duration::from_millis(30)).await;
        }
        Ok(())
    }
}
