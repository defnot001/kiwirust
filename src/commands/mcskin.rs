use std::fmt::Display;

use anyhow::Context;

use crate::{util::mojang::MojangAPI, Context as AppContext};

#[derive(Debug, poise::ChoiceParameter)]
enum SkinPositionChoice {
    Default,
    Marching,
    Walking,
    Crouching,
    Crossed,
    CrissCross,
    Ultimate,
    Cheering,
    Relaxing,
    Trudging,
    Cowering,
    Pointing,
    Lunging,
    Isometric,
    Head,
    Skin,
}

#[derive(Debug, poise::ChoiceParameter)]
enum RenderTypeChoice {
    Full,
    Bust,
    Face,
    Head,
    Default,
    Processed,
}

impl Display for SkinPositionChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkinPositionChoice::Default => write!(f, "default"),
            SkinPositionChoice::Marching => write!(f, "marching"),
            SkinPositionChoice::Walking => write!(f, "walking"),
            SkinPositionChoice::Crouching => write!(f, "crouching"),
            SkinPositionChoice::Crossed => write!(f, "crossed"),
            SkinPositionChoice::CrissCross => write!(f, "criss_cross"),
            SkinPositionChoice::Ultimate => write!(f, "ultimate"),
            SkinPositionChoice::Cheering => write!(f, "cheering"),
            SkinPositionChoice::Relaxing => write!(f, "relaxing"),
            SkinPositionChoice::Trudging => write!(f, "trudging"),
            SkinPositionChoice::Cowering => write!(f, "cowering"),
            SkinPositionChoice::Pointing => write!(f, "pointing"),
            SkinPositionChoice::Lunging => write!(f, "lunging"),
            SkinPositionChoice::Isometric => write!(f, "isometric"),
            SkinPositionChoice::Head => write!(f, "head"),
            SkinPositionChoice::Skin => write!(f, "skin"),
        }
    }
}

impl Display for RenderTypeChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderTypeChoice::Full => write!(f, "full"),
            RenderTypeChoice::Bust => write!(f, "bust"),
            RenderTypeChoice::Face => write!(f, "face"),
            RenderTypeChoice::Head => write!(f, "head"),
            RenderTypeChoice::Default => write!(f, "default"),
            RenderTypeChoice::Processed => write!(f, "processed"),
        }
    }
}

/// Get the minecraft skin of a player.
#[poise::command(slash_command, guild_only = true)]
pub async fn mcskin(
    ctx: AppContext<'_>,
    #[description = "The player to get the skin of."] player_name: String,
    #[description = "The position of the player in the skin."] position: SkinPositionChoice,
    #[description = "The type of the image. Full is the default."] render_type: RenderTypeChoice,
) -> anyhow::Result<()> {
    ctx.defer().await?;

    if player_name.trim().is_empty() {
        return Err(anyhow::anyhow!("Player name cannot be empty."));
    }

    let mojang_profile = MojangAPI::get_profile(&player_name).await?;

    match ctx.say(format!("{:#?}", mojang_profile)).await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!("Failed to send message: {:?}", e);
            return Err(e).context("Failed to send message");
        }
    }
}
