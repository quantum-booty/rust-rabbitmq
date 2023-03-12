use anyhow::Result;
use prost::Message;
use tokio::time;
use tracing::info;

use crate::{
    items::{shirt::Size, Shirt},
    message_queue::{rabbit::RabbitClient, MessageQueuePublisher},
};

pub async fn test_protobuf_generate(rabbit_client: RabbitClient) -> Result<()> {
    let queue = "test_protobuf_queue_name";
    let publisher = rabbit_client.get_publisher(queue).await?;
    for i in 0.. {
        let message = Shirt {
            color: format!("yayaya {i}"),
            size: Size::Small.into(),
        };

        info!("sending message {message:?}");
        publisher.publish(message.encode_to_vec()).await?;
        time::sleep(time::Duration::from_millis(1)).await;
    }
    Ok(())
}
