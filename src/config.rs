use std::{
    fmt::{Debug, Display},
    num::NonZeroU64,
};

use anyhow::Context;
use poise::serenity_prelude as serenity;
use serde::Deserialize;

#[derive(Debug, Copy, Clone, poise::ChoiceParameter)]
#[repr(usize)]
pub enum ServerChoice {
    Smp,
    Cmp,
    Cmp2,
    Copy,
    Snapshots,
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

impl TryFrom<&ServerConfig> for ServerChoice {
    type Error = anyhow::Error;

    fn try_from(server_config: &ServerConfig) -> anyhow::Result<Self> {
        match server_config.server_name.as_str() {
            "smp" => Ok(Self::Smp),
            "cmp" => Ok(Self::Cmp),
            "cmp2" => Ok(Self::Cmp2),
            "copy" => Ok(Self::Copy),
            "snapshots" => Ok(Self::Snapshots),
            _ => Err(anyhow::anyhow!(
                "Unknown server choice: {}",
                server_config.server_name
            )),
        }
    }
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

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_file = std::fs::File::open("config.json")?;
        let reader = std::io::BufReader::new(config_file);

        serde_json::from_reader(reader).context("Failed to parse config.json file")
    }
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

#[derive(Clone)]
pub struct MinecraftConfig {
    configs: [ServerConfig; 5],
}

impl MinecraftConfig {
    pub fn get(&self, choice: ServerChoice) -> &ServerConfig {
        &self.configs[choice as usize]
    }

    pub fn get_owned(self, choice: ServerChoice) -> ServerConfig {
        self.configs.into_iter().nth(choice as usize).unwrap()
    }
}

macro_rules! declare_config_getters {
    ($getter_name: ident, $owned_getter_name: ident, $choice: ident) => {
        impl MinecraftConfig {
            pub fn $getter_name(&self) -> &ServerConfig {
                self.get(ServerChoice::$choice)
            }

            pub fn $owned_getter_name(self) -> ServerConfig {
                self.get_owned(ServerChoice::$choice)
            }
        }
    };
}

declare_config_getters!(smp, smp_owned, Smp);
declare_config_getters!(cmp, cmp_owned, Cmp);
declare_config_getters!(cmp2, cmp2_owned, Cmp2);
declare_config_getters!(copy, copy_owned, Copy);
declare_config_getters!(snapshots, snapshots_owned, Snapshots);

impl Debug for MinecraftConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MinecraftConfig")
            .field("smp", &self.configs[ServerChoice::Smp as usize])
            .field("cmp", &self.configs[ServerChoice::Cmp as usize])
            .field("cmp2", &self.configs[ServerChoice::Cmp2 as usize])
            .field("copy", &self.configs[ServerChoice::Copy as usize])
            .field("snapshots", &self.configs[ServerChoice::Snapshots as usize])
            .finish()
    }
}

impl IntoIterator for MinecraftConfig {
    type Item = ServerConfig;
    type IntoIter = std::array::IntoIter<Self::Item, 5>;

    fn into_iter(self) -> Self::IntoIter {
        self.configs.into_iter()
    }
}

impl<'a> IntoIterator for &'a MinecraftConfig {
    type Item = &'a ServerConfig;
    type IntoIter = std::slice::Iter<'a, ServerConfig>;

    fn into_iter(self) -> Self::IntoIter {
        self.configs.iter()
    }
}

impl<'de> Deserialize<'de> for MinecraftConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct DeserializeMinecraftConfig {
            smp: ServerConfig,
            cmp: ServerConfig,
            cmp2: ServerConfig,
            copy: ServerConfig,
            snapshots: ServerConfig,
        }

        let DeserializeMinecraftConfig {
            smp,
            cmp,
            cmp2,
            copy,
            snapshots,
        } = DeserializeMinecraftConfig::deserialize(deserializer)?;

        Ok(MinecraftConfig {
            configs: [smp, cmp, cmp2, copy, snapshots],
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub server_name: String,
    pub host: String,
    pub port: u16,
    pub rcon_port: u16,
    pub rcon_password: String,
    pub panel_id: String,
    pub operator: bool,
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
