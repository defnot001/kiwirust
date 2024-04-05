use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MojangProfile {
    pub id: String,
    pub name: String,
}

pub struct MojangAPI;

impl MojangAPI {
    pub async fn get_profile(username: impl AsRef<str>) -> anyhow::Result<MojangProfile> {
        reqwest::get(format!(
            "https://api.mojang.com/users/profiles/minecraft/${}",
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
}
