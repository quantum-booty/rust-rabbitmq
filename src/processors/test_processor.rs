use crate::processors::Processor;
use amqprs::channel::Channel;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::time;
use tracing::info;

use crate::{
    message_queue::{rabbit::RabbitMessageQueueReceiver, MessageQueueReceiver},
    message_types::TestMessage,
};

pub struct TestProcessor {
    receiver: RabbitMessageQueueReceiver,
}

impl TestProcessor {
    pub async fn new(channel: Arc<Channel>) -> Result<Self> {
        let queue_name = "edge.do_something_processor";

        info!("Starting process {queue_name}");
        let receiver =
            RabbitMessageQueueReceiver::new(channel.clone(), queue_name, "yaya_processor").await?;

        Ok(Self { receiver })
    }
}

#[async_trait]
impl Processor for TestProcessor {
    async fn run(&mut self) -> Result<()> {
        while let Some(message) = self.receiver.receive().await? {
            let deliver = message.deliver.unwrap();
            let message: TestMessage = serde_json::from_slice(&message.content.unwrap())?;

            info!("received a message {:?}", message);

            info!("processing message {:?}", message);
            // need ability to batch process messages
            // the processor potentially need sql connection, blob storage connection string, redis connection, configurations, etc
            // there should be separation of the queuing logic and processing logic
            // once_cell global configuration

            // if doing batch processing, can set multple = true to ack multiple items up to the delivery tag
            self.receiver.ack(deliver).await?;

            time::sleep(time::Duration::from_millis(50)).await;
        }

        Ok(())
    }
}
