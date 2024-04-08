use std::{fmt::Display, num::NonZeroU64};

use anyhow::Context;
use poise::serenity_prelude as serenity;
use serde::Deserialize;

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
    pub channels: ChannelConfig,
    pub categories: CategoryConfig,
    pub pterodactyl: PterodactylConfig,
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
    pub admin: serenity::RoleId,
    pub member: serenity::RoleId,
    pub members: serenity::RoleId,
    pub pingpong: serenity::RoleId,
    pub trial: serenity::RoleId,
    pub kiwi_inc: serenity::RoleId,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChannelConfig {
    pub member_log: serenity::ChannelId,
    pub mod_log: serenity::ChannelId,
    pub bot_log: serenity::ChannelId,
    pub invite: serenity::ChannelId,
    pub resources: serenity::ChannelId,
    pub server_info: serenity::ChannelId,
    pub todo: serenity::ChannelId,
    pub todo_log: serenity::ChannelId,
    pub application: serenity::ChannelId,
    pub application_voting: serenity::ChannelId,
    pub member_general: serenity::ChannelId,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CategoryConfig {
    pub application: serenity::ChannelId,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PterodactylConfig {
    pub url: String,
    pub api_key: String,
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

impl MinecraftConfig {
    pub fn server_config(&self, choice: &ServerChoice) -> &ServerConfig {
        match choice {
            ServerChoice::Smp => &self.smp,
            ServerChoice::Cmp => &self.cmp,
            ServerChoice::Cmp2 => &self.cmp2,
            ServerChoice::Copy => &self.copy,
            ServerChoice::Snapshots => &self.snapshots,
        }
    }
}
