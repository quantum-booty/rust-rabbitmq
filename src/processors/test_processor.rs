use crate::processors::Processor;
use std::sync::Arc;

use amqprs::channel::{BasicAckArguments, Channel};
use anyhow::Result;
use async_trait::async_trait;
use tokio::time;
use tracing::info;

use crate::{
    message_queue::{rabbit::RabbitMessageQueueReceiver, MessageQueueReceiver},
    message_types::TestMessage,
};

pub struct TestProcessor {
    receiver: RabbitMessageQueueReceiver,
    channel: Arc<Channel>,
}

impl TestProcessor {
    pub async fn new(channel: Arc<Channel>) -> Result<Self> {
        let queue_name = "edge.do_something_processor";

        info!("Starting process {queue_name}");
        let receiver =
            RabbitMessageQueueReceiver::new(&channel, queue_name, "yaya_processor").await?;

        Ok(Self { receiver, channel })
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
            self.channel
                .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
                .await?;

            time::sleep(time::Duration::from_millis(50)).await;
        }

        Ok(())
    }
}