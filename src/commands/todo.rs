use std::{fmt::Display, str::FromStr};

use crate::Context as AppContext;

#[derive(Debug, poise::ChoiceParameter)]
pub enum TodoChoice {
    Creative,
    Survival,
}

impl Display for TodoChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Creative => write!(f, "creative"),
            Self::Survival => write!(f, "survival")
        }

    }
}

impl FromStr for TodoChoice {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "creative" => Ok(TodoChoice::Creative),
            "survival" => Ok(TodoChoice::Survival),
            _ => anyhow::bail!("{} is not a valid todo choice", s)
        }
    }
}

/// Add, update or complete a todo item.
#[poise::command(
    slash_command,
    guild_only = true,
    subcommands("add", "update", "complete"),
    subcommand_required
)]
pub async fn todo(_: AppContext<'_>) -> anyhow::Result<()> {
    Ok(())
}

#[poise::command(slash_command, guild_only = true)]
pub async fn add(
    ctx: AppContext<'_>,
    #[description = "Choose wether the todo is related to survival or creative gameplay."]
    todo_type: TodoChoice,
    #[description = "The title of the todo item."]
    title: String,
) -> anyhow::Result<()> {
    Ok(())
}

#[poise::command(slash_command, guild_only = true)]
pub async fn update(ctx: AppContext<'_>) -> anyhow::Result<()> {
    Ok(())
}

#[poise::command(slash_command, guild_only = true)]
pub async fn complete(ctx: AppContext<'_>) -> anyhow::Result<()> {
    Ok(())
}
