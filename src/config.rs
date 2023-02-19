use anyhow::Result;
use config::{Config, ConfigError, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Rabbit {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct Configs {
    pub database: Database,
    pub rabbit: Rabbit,
}

impl Configs {
    pub fn new(_environment: &str) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(Environment::default().separator("_"))
            .build()?;
        s.try_deserialize()
    }
}
