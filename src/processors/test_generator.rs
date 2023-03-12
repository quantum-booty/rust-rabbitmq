use anyhow::Result;
use tokio::time;
use tracing::info;

use crate::{
    message_queue::{rabbit::RabbitClient, MessageQueuePublisher},
    message_types::TestMessage,
};

pub async fn test_generate(rabbit_client: RabbitClient, wait_ms: u64) -> Result<()> {
    let queue = "test_queue_name";
    let publisher = rabbit_client.get_publisher(queue).await?;
    for i in 0.. {
        let message = TestMessage {
            publisher: "example generator".to_string(),
            data: format!("hello world {i}"),
        };

        info!("sending message {message}");
        publisher.publish(serde_json::to_vec(&message)?).await?;
        time::sleep(time::Duration::from_millis(wait_ms)).await;
    }
    Ok(())
}
