use std::{fs::File, io::BufReader};

use lazy_static::lazy_static;
use serde::Deserialize;
use serenity::model::prelude::GuildId;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub bot: BotConfig,
    pub guild: GuildConfig,
}

#[derive(Deserialize, Debug)]
pub struct BotConfig {
    pub bot_token: String,
}

#[derive(Deserialize, Debug)]
pub struct GuildConfig {
    pub guild_id: GuildId,
}

lazy_static! {
    static ref CONFIG: Config = Config::load();
}

impl Config {
    fn load() -> Config {
        let file = File::open("config.json").expect("Failed to open the config.file");
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).expect("Failed to parse the config file contents.")
    }

    pub fn get() -> &'static Self {
        &CONFIG
    }
}
