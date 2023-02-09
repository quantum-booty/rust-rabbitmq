use amqprs::channel::Channel;
use anyhow::Result;
use std::sync::Arc;
use tokio::time;

use crate::{
    message_queue::{
        rabbit::{declare_queue, RabbitQueueMessagePublisher, EXCHANGE},
        MessageQueuePublisher,
    },
    message_types::TestMessage,
};

pub async fn test_generate(channel: Arc<Channel>) -> Result<()> {
    let queue_name = "edge.do_something_processor";
    declare_queue(&channel, EXCHANGE, queue_name, 1).await?;
    let publisher = RabbitQueueMessagePublisher::new(channel.clone(), EXCHANGE, queue_name);
    do_run(publisher).await
}

async fn do_run(publisher: impl MessageQueuePublisher) -> Result<()> {
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
