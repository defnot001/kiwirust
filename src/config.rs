use std::num::NonZeroU64;

use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bot: BotConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct BotConfig {
    pub token: String,
    pub client_id: NonZeroU64,
    pub guild_id: NonZeroU64,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_file = std::fs::File::open("config.json")?;
        let reader = std::io::BufReader::new(config_file);

        serde_json::from_reader(reader).context("Failed to parse config.json file")
    }
}
