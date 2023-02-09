use amqprs::channel::Channel;
use anyhow::Result;
use std::sync::Arc;
use tokio::time;
use tracing::info;

use crate::{
    message_queue::{rabbit::RabbitMessageQueueReceiver, MessageQueueReceiver},
    message_types::TestMessage,
};

pub async fn test_process(channel: Arc<Channel>) -> Result<()> {
    let queue_name = "edge.do_something_processor";

    info!("Starting process {queue_name}");
    let mut receiver =
        RabbitMessageQueueReceiver::new(channel.clone(), queue_name, "yaya_processor").await?;

    while let Some((message, ack_id)) = receiver.receive().await {
        let message_data: TestMessage = serde_json::from_slice(&message.content.unwrap())?;
        info!("received a message {:?}", message_data);

        do_run(message_data);

        // need ability to batch process messages
        // the processor potentially need sql connection, blob storage connection string, redis connection, configurations, etc
        // there should be separation of the queuing logic and processing logic
        // once_cell global configuration

        // if doing batch processing, can set multple = true to ack multiple items up to the delivery tag
        receiver.ack(ack_id).await?;

        time::sleep(time::Duration::from_millis(50)).await;
    }

    Ok(())
}

fn do_run(message_data: TestMessage) {
    info!("processing message {:?}", message_data);
    info!("processed message {:?}", message_data);
}
