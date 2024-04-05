use std::path::PathBuf;

use anyhow::Context;
use poise::CreateReply;

use crate::{util::builder::default_embed, Context as AppContext};

#[derive(Debug, poise::ChoiceParameter)]
enum HelpChoice {
    Mobswitches,
    Building,

    #[name = "Bed Bot"]
    BedBot,

    #[name = "10gt Raid Farm"]
    Raid,

    #[name = "Mushroom Farms"]
    Mushroom,
}

/// Get information on how to use things on SMP.
#[poise::command(slash_command, guild_only = true)]
pub async fn help(
    ctx: AppContext<'_>,
    #[description = "The thing you want to get information about."] thing: HelpChoice,
) -> anyhow::Result<()> {
    ctx.defer().await?;

    let content = read_help_file(&thing).await?;
    let embed = default_embed(&ctx.author()).description(content);

    match ctx.send(CreateReply::default().embed(embed)).await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!("Failed to send message: {:?}", e);
            return Err(e).context("Failed to send message");
        }
    }
}

async fn read_help_file(choice: &HelpChoice) -> anyhow::Result<String> {
    let help_dir_path = PathBuf::from("assets/help");
    let file_path = match choice {
        HelpChoice::Mobswitches => help_dir_path.join("mobswitch.md"),
        HelpChoice::Building => help_dir_path.join("building.md"),
        HelpChoice::BedBot => help_dir_path.join("bedbot.md"),
        HelpChoice::Raid => help_dir_path.join("raid.md"),
        HelpChoice::Mushroom => help_dir_path.join("mushroom.md"),
    };

    match tokio::fs::read_to_string(file_path).await {
        Ok(content) => Ok(content),
        Err(e) => {
            tracing::error!("Error reading help file: {}", e);
            Err(e.into())
        }
    }
}
