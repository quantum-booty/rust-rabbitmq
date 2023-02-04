use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct TestMessage {
    pub publisher: String,
    pub data: String,
}
