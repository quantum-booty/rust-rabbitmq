pub mod cli;
pub mod config;
pub mod log;
pub mod message_queue;
pub mod message_types;
pub mod processors;

pub mod items {
    include!(concat!(env!("OUT_DIR"), "/items.rs"));
}
