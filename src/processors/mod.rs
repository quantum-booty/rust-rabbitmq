pub mod test_generator;
pub mod test_processor;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Processor {
    async fn run(&mut self) -> Result<()>;
}
