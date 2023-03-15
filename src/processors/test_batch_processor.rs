use anyhow::Result;
use std::time::Duration;
use tracing::info;

use crate::{
    message_queue::{rabbit::RabbitClient, ChunkReceiver},
    message_types::TestMessage,
};

pub async fn test_batch_process(rabbit_client: RabbitClient) -> Result<()> {
    let queue = "test_queue_name";
    info!("Starting process {queue}");

    let mut receiver = rabbit_client
        .get_chunk_receiver(
            queue,
            "test_batch_processor",
            100,
            10,
            Duration::from_secs(1),
        )
        .await?;

    let mut batch_number = 0;
    while let Some(messages) = receiver.receive().await {
        batch_number += 1;
        for message in messages {
            let message_data: TestMessage = message.json_deserialise()?;
            info!("received a message {:?} from batch {}", message_data, batch_number);

            do_run(message_data);

            receiver.ack(&message, false).await?;
        }
    }

    Ok(())
}

fn do_run(message_data: TestMessage) {
    info!("processing message {:?}", message_data);
    info!("processed message {:?}", message_data);
}
