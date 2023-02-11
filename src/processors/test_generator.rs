use anyhow::Result;
use tokio::time;

use crate::{
    message_queue::{rabbit::RabbitClient, MessageQueuePublisher},
    message_types::TestMessage,
};

pub async fn test_generate(rabbit_client: RabbitClient) -> Result<()> {
    let queue = "test_queue_name";
    let publisher = rabbit_client.get_publisher(queue).await?;
    for i in 0.. {
        let message = TestMessage {
            publisher: "example generator".to_string(),
            data: format!("hello world {i}"),
        };

        publisher.publish(message).await?;
        time::sleep(time::Duration::from_millis(30)).await;
    }
    Ok(())
}
