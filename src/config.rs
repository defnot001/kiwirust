use std::{fmt::Display, num::NonZeroU64};

use anyhow::Context;
use poise::serenity_prelude as serenity;
use serde::Deserialize;
use serde_json::ser::Formatter;

#[derive(Debug, poise::ChoiceParameter)]
pub enum ServerChoice {
    Smp,
    Cmp,
    Cmp2,
    Copy,
    Snapshots,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub bot: BotConfig,
    pub database: DatabaseConfig,
    pub minecraft: MinecraftConfig,
    pub roles: RoleConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BotConfig {
    pub token: String,
    pub client_id: NonZeroU64,
    pub guild_id: serenity::GuildId,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MinecraftConfig {
    pub host: String,
    pub smp: ServerConfig,
    pub cmp: ServerConfig,
    pub cmp2: ServerConfig,
    pub copy: ServerConfig,
    pub snapshots: ServerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub rcon_port: u16,
    pub rcon_password: String,
    pub panel_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RoleConfig {
    admin: serenity::RoleId,
    member: serenity::RoleId,
    members: serenity::RoleId,
    pingpong: serenity::RoleId,
    trial: serenity::RoleId,
    kiwi_inc: serenity::RoleId,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_file = std::fs::File::open("config.json")?;
        let reader = std::io::BufReader::new(config_file);

        serde_json::from_reader(reader).context("Failed to parse config.json file")
    }
}

impl Display for ServerChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ServerChoice::Smp => write!(f, "SMP"),
            ServerChoice::Cmp => write!(f, "CMP"),
            ServerChoice::Cmp2 => write!(f, "CMP2"),
            ServerChoice::Copy => write!(f, "Copy"),
            ServerChoice::Snapshots => write!(f, "Snapshots"),
        }
    }
}
