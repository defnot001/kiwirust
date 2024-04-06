use std::fmt::Display;

use anyhow::Context;
use poise::{serenity_prelude as serenity, CreateReply};

use crate::{error::respond_error, util::mojang::MojangAPI, Context as AppContext};

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

#[derive(Debug, PartialEq, poise::ChoiceParameter)]
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
        ctx.say("Player name cannot be empty.").await?;
        return Ok(());
    }

    let allowed_render_types = get_allowed_render_types(&position);

    if !allowed_render_types.contains(&render_type) {
        ctx.say(format!(
            "Render type {} is not allowed for position {}! This position only allows {}.",
            render_type,
            position,
            allowed_render_types
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))
        .await?;
        return Ok(());
    }

    let mojang_profile = match MojangAPI::get_profile(&player_name).await {
        Ok(profile) => profile,
        Err(e) => {
            return respond_error(format!("Failed to get profile for {player_name}"), e, &ctx)
                .await;
        }
    };

    let skin_url = format!(
        "https://starlightskins.lunareclipse.studio/render/{position}/{}/{render_type}",
        mojang_profile.id,
    );

    let res = match reqwest::get(&skin_url).await {
        Ok(res) => res,
        Err(e) => {
            return respond_error("Failed to get skin", e, &ctx).await;
        }
    };

    let Some(content_type) = res.headers().get("content-type") else {
        ctx.say("Header content-type not found").await?;
        return Err(anyhow::anyhow!("Header content-type not found"));
    };

    let content_type = content_type
        .to_str()
        .context("Failed to parse content-type")?
        .to_string();

    if !content_type.starts_with("image/") {
        return Err(anyhow::anyhow!("Content is not am image"));
    }

    if !res.status().is_success() {
        return Err(anyhow::anyhow!(
            "Skin API returned Status: {}",
            res.status()
        ));
    }

    let content = res.bytes().await.context("Failed to get content")?;

    let file_extension = match content_type.as_str() {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/webp" => "webp",
        _ => {
            anyhow::bail!("Unsupported content type: {}", content_type);
        }
    };

    let reply = CreateReply::default().attachment(serenity::CreateAttachment::bytes(
        content,
        format!("{}.{}", &player_name, file_extension),
    ));

    ctx.send(reply).await?;

    Ok(())
}

fn get_allowed_render_types(pos: &SkinPositionChoice) -> Vec<RenderTypeChoice> {
    match pos {
        SkinPositionChoice::Default
        | SkinPositionChoice::Marching
        | SkinPositionChoice::Walking
        | SkinPositionChoice::Crouching
        | SkinPositionChoice::Crossed
        | SkinPositionChoice::CrissCross
        | SkinPositionChoice::Ultimate
        | SkinPositionChoice::Cheering
        | SkinPositionChoice::Relaxing
        | SkinPositionChoice::Trudging
        | SkinPositionChoice::Cowering
        | SkinPositionChoice::Pointing
        | SkinPositionChoice::Lunging => vec![
            RenderTypeChoice::Full,
            RenderTypeChoice::Bust,
            RenderTypeChoice::Face,
        ],
        SkinPositionChoice::Isometric => vec![
            RenderTypeChoice::Full,
            RenderTypeChoice::Bust,
            RenderTypeChoice::Face,
            RenderTypeChoice::Head,
        ],
        SkinPositionChoice::Head => vec![RenderTypeChoice::Full],
        SkinPositionChoice::Skin => vec![RenderTypeChoice::Default, RenderTypeChoice::Processed],
    }
}
