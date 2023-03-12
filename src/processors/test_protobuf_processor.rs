use anyhow::Result;
use tokio::time;
use tracing::info;

use crate::{
    message_queue::{rabbit::RabbitClient, MessageQueueReceiver}, items::Shirt,
};

pub async fn test_protobuf_process(rabbit_client: RabbitClient) -> Result<()> {
    let queue = "test_protobuf_queue_name";
    info!("Starting process {queue}");

    let mut receiver = rabbit_client
        .get_receiver(queue, "test_protobuf_processor", 1)
        .await?;

    while let Some(message) = receiver.receive().await {
        let message_data: Shirt = message.protobuf_deserialise()?;
        info!("received a message {:?}", message_data);

        do_run(message_data);

        receiver.ack(&message).await?;

        time::sleep(time::Duration::from_millis(1)).await;
    }

    Ok(())
}

fn do_run(message_data: Shirt) {
    info!("processing message {:?}", message_data);
    info!("processed message {:?}", message_data);
}
