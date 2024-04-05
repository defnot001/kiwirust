use anyhow::Context;
use poise::CreateReply;
use serde::Deserialize;
use serenity::all::CreateAttachment;

use crate::Context as AppContext;

#[derive(Debug, poise::ChoiceParameter)]
pub enum AnimalChoice {
    Fox,
    Cat,
    Dog,
}

#[derive(Debug, Deserialize)]
pub struct FoxResponse {
    image: String,
    link: String,
}

/// Get random pictures of animals.
#[poise::command(slash_command, guild_only = true)]
pub async fn animal(
    ctx: AppContext<'_>,
    #[description = "What kind of animal do you want to see?"] animal: AnimalChoice,
) -> anyhow::Result<()> {
    ctx.defer().await?;

    let image_link = match get_image_link(animal).await {
        Ok(link) => link,
        Err(e) => {
            tracing::error!("Failed to get image link: {:?}", e);
            return Err(e).context("Failed to get image link");
        }
    };

    let attachment = match CreateAttachment::url(ctx, image_link.as_str()).await {
        Ok(attachment) => attachment,
        Err(e) => {
            tracing::error!("Failed to create attachment: {:?}", e);
            return Err(e).context("Failed to create attachment");
        }
    };

    match ctx
        .send(CreateReply::default().attachment(attachment))
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!("Failed to send message: {:?}", e);
            return Err(e).context("Failed to send message");
        }
    }
}

async fn get_image_link(animal: AnimalChoice) -> anyhow::Result<String> {
    let api_url = match animal {
        AnimalChoice::Fox => "https://randomfox.ca/floof/",
        AnimalChoice::Cat => "https://api.thecatapi.com/v1/images/search",
        AnimalChoice::Dog => "https://api.thedogapi.com/v1/images/search",
    };

    let image_url = match animal {
        AnimalChoice::Fox => {
            let response = reqwest::get(api_url)
                .await?
                .json::<serde_json::Value>()
                .await?;

            match response["image"].as_str() {
                Some(link) => Ok(link.to_string()),
                None => Err(anyhow::anyhow!("Failed to get image URL from the API.")),
            }
        }
        AnimalChoice::Cat | AnimalChoice::Dog => {
            let response = reqwest::get(api_url)
                .await?
                .json::<Vec<serde_json::Value>>()
                .await?;

            match response[0]["url"].as_str() {
                Some(url) => Ok(url.to_string()),
                None => Err(anyhow::anyhow!("Failed to get image URL from the API.")),
            }
        }
    };

    image_url
}
