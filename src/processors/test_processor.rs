use anyhow::Result;
use tokio::time;
use tracing::info;

use crate::{
    message_queue::{rabbit::RabbitClient, MessageQueueReceiver},
    message_types::TestMessage,
};

pub async fn test_process(rabbit_client: RabbitClient) -> Result<()> {
    let queue = "test_queue_name";
    info!("Starting process {queue}");

    let mut receiver = rabbit_client.get_receiver(queue, "test_processor").await?;

    while let Some(message) = receiver.receive().await {
        let message_data = message.json_deserialise::<TestMessage>()?;
        info!("received a message {:?}", message_data);

        do_run(message_data);

        // need ability to batch process messages
        // the processor potentially need sql connection, blob storage connection string, redis connection, configurations, etc
        // there should be separation of the queuing logic and processing logic
        // once_cell global configuration

        // if doing batch processing, can set multple = true to ack multiple items up to the delivery tag
        receiver.ack(&message).await?;

        time::sleep(time::Duration::from_millis(50)).await;
    }

    Ok(())
}

fn do_run(message_data: TestMessage) {
    info!("processing message {:?}", message_data);
    info!("processed message {:?}", message_data);
}
