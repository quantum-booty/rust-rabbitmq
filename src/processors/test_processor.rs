use anyhow::Result;
use tokio::time;
use tracing::info;

use crate::{
    message_queue::{rabbit::RabbitClient, MessageQueueReceiver},
    message_types::TestMessage,
};

pub async fn test_process(rabbit_client: RabbitClient, wait_ms: u64) -> Result<()> {
    let queue = "test_queue_name";
    info!("Starting process {queue}");

    let mut receiver = rabbit_client
        .get_receiver(queue, "test_processor", 1)
        .await?;

    while let Some(message) = receiver.receive().await {
        let message_data: TestMessage = message.json_deserialise()?;
        info!("received a message {:?}", message_data);

        do_run(message_data);

        receiver.ack(&message).await?;

        time::sleep(time::Duration::from_millis(wait_ms)).await;
    }

    Ok(())
}

fn do_run(message_data: TestMessage) {
    info!("processing message {:?}", message_data);
    info!("processed message {:?}", message_data);
}
