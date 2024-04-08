use anyhow::Context;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct MojangProfile {
    pub id: Uuid,
    pub name: String,
}

pub struct MojangAPI;

impl MojangAPI {
    pub async fn get_profile_from_username(
        username: impl AsRef<str>,
    ) -> anyhow::Result<MojangProfile> {
        reqwest::get(format!(
            "https://api.mojang.com/users/profiles/minecraft/{}",
            username.as_ref()
        ))
        .await
        .context(format!(
            "Failed to get profile for {} from the mojang API",
            username.as_ref()
        ))?
        .json::<MojangProfile>()
        .await
        .context(format!(
            "Failed to parse profile for {} from the mojang API",
            username.as_ref()
        ))
    }

    pub async fn get_profiles(usernames: Vec<String>) -> anyhow::Result<Vec<MojangProfile>> {
        if usernames.len() > 10 {
            anyhow::bail!("You can only request 10 profiles at a time")
        }

        reqwest::Client::new()
            .post("https://api.minecraftservices.com/minecraft/profile/lookup/bulk/byname")
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&usernames)?)
            .send()
            .await?
            .json::<Vec<MojangProfile>>()
            .await
            .context("Failed to parse profiles from the mojang API")
    }

    pub async fn get_profile_from_uuid(uuid: &Uuid) -> anyhow::Result<MojangProfile> {
        reqwest::get(format!(
            "https://sessionserver.mojang.com/session/minecraft/profile/{}",
            uuid
        ))
        .await?
        .json::<MojangProfile>()
        .await
        .context(format!("Failed to parse response for uuid {}", uuid))
    }
}
