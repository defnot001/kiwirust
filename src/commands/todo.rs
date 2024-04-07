use std::{fmt::Display, str::FromStr};

use serenity::all::{
    CreateEmbed, CreateEmbedFooter, CreateMessage, GetMessages, GuildChannel, Timestamp,
};

use crate::{
    database::model::todo::{CreateTodo, Todo, TodoModelController},
    error::respond_error,
    util::format::{fdisplay, inline_code},
    Context as AppContext,
};

#[derive(Debug, Clone, poise::ChoiceParameter)]
pub enum TodoChoice {
    Creative,
    Survival,
}

impl Display for TodoChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Creative => write!(f, "creative"),
            Self::Survival => write!(f, "survival"),
        }
    }
}

impl FromStr for TodoChoice {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "creative" => Ok(TodoChoice::Creative),
            "survival" => Ok(TodoChoice::Survival),
            _ => anyhow::bail!("{} is not a valid todo choice", s),
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
    #[description = "The title of the todo item."] title: String,
) -> anyhow::Result<()> {
    ctx.defer_ephemeral().await?;

    if title.trim().is_empty() {
        ctx.say("Title cannot be empty!").await?;
        return Ok(());
    }

    if let Err(e) = TodoModelController::create(
        &ctx.data().db_pool,
        CreateTodo::new(title.clone(), todo_type.clone(), ctx.author().id),
    )
    .await
    {
        return respond_error("Failed to create todo in the database", e, &ctx).await;
    };

    let description = format!(
        "{} added a new todo item for the {todo_type} list: {}",
        fdisplay(ctx.author()),
        inline_code(title)
    );

    if let Err(e) = send_todolog(description, &ctx).await {
        tracing::error!("Failed to send todo log: {e}")
    }

    if let Err(e) = update_todo_embed(&ctx).await {
        tracing::error!("Failed to update the todo embeds: {e}")
    }

    ctx.say("Successfully added todo item to the database.")
        .await?;

    Ok(())
}

#[poise::command(slash_command, guild_only = true)]
pub async fn update(
    ctx: AppContext<'_>,
    #[description = "The title of the todo item you want to change."] old_title: String,
    #[description = "The title you want to change it to."] new_title: String,
) -> anyhow::Result<()> {
    if old_title.trim().is_empty() || new_title.trim().is_empty() {
        ctx.say("Titles cannot be empty!").await?;
        return Ok(());
    }

    if let Err(e) =
        TodoModelController::update_title(&ctx.data().db_pool, old_title.clone(), new_title.clone())
            .await
    {
        return respond_error("Failed to update todo in the database", e, &ctx).await;
    };

    let description = format!(
        "{} updated a todo item: {} to {}",
        fdisplay(ctx.author()),
        inline_code(old_title),
        inline_code(new_title)
    );

    if let Err(e) = send_todolog(description, &ctx).await {
        tracing::error!("Failed to send todo log: {e}")
    }

    if let Err(e) = update_todo_embed(&ctx).await {
        tracing::error!("Failed to update the todo embeds: {e}")
    }

    ctx.say("Successfully updated todo item in the database.")
        .await?;

    Ok(())
}

#[poise::command(slash_command, guild_only = true)]
pub async fn complete(
    ctx: AppContext<'_>,
    #[description = "The title of the todo you want to complete."] title: String,
) -> anyhow::Result<()> {
    if title.trim().is_empty() {
        ctx.say("Title cannot be empty!").await?;
        return Ok(());
    }

    if let Err(e) = TodoModelController::delete_by_title(&ctx.data().db_pool, title.clone()).await {
        return respond_error("Failed to delete todo from the database", e, &ctx).await;
    }

    let description = format!(
        "{} completed a todo item: {}",
        fdisplay(ctx.author()),
        inline_code(title)
    );

    if let Err(e) = send_todolog(description, &ctx).await {
        tracing::error!("Failed to send todo log: {e}")
    }

    if let Err(e) = update_todo_embed(&ctx).await {
        tracing::error!("Failed to update the todo embeds: {e}")
    }

    ctx.say("Successfully cmpleted todo item in the database.")
        .await?;

    Ok(())
}

async fn send_todolog(description: impl Into<String>, ctx: &AppContext<'_>) -> anyhow::Result<()> {
    let Some(guild) = ctx.partial_guild().await else {
        anyhow::bail!("Cannot find the guild this interaction was created in")
    };

    let Some(channel) = ctx
        .data()
        .config
        .channels
        .todo_log
        .to_channel(&ctx)
        .await?
        .guild()
    else {
        anyhow::bail!("Cannot find todo log channel")
    };

    let footer = CreateEmbedFooter::new(ctx.author().name.to_string()).icon_url(
        ctx.author()
            .avatar_url()
            .unwrap_or(ctx.author().default_avatar_url()),
    );

    let embed = CreateEmbed::new()
        .title(format!("{} Todo Log", guild.name))
        .color(3_517_048)
        .footer(footer)
        .timestamp(Timestamp::now())
        .description(description);

    channel
        .send_message(&ctx, CreateMessage::new().add_embed(embed))
        .await?;

    Ok(())
}

async fn update_todo_embed(ctx: &AppContext<'_>) -> anyhow::Result<()> {
    let Some(todo_channel) = ctx
        .data()
        .config
        .channels
        .todo
        .to_channel(ctx)
        .await?
        .guild()
    else {
        anyhow::bail!("Cannot find todo channel")
    };

    let all_todos = TodoModelController::get_all(&ctx.data().db_pool).await?;

    clear_channel(&todo_channel, ctx).await?;

    let survival_embed = CreateEmbed::new()
        .title("Survival Todo List")
        .colour(3_866_688)
        .description(display_todos(all_todos.survival))
        .timestamp(Timestamp::now());

    let creative_embed = CreateEmbed::new()
        .title("Creative Todo List")
        .colour(5_243_182)
        .description(display_todos(all_todos.creative))
        .timestamp(Timestamp::now());

    todo_channel
        .send_message(
            ctx,
            CreateMessage::new().add_embeds(vec![survival_embed, creative_embed]),
        )
        .await?;

    Ok(())
}

fn display_todos(todos: Vec<Todo>) -> String {
    todos
        .into_iter()
        .map(|todo| format!("• {}", todo.title))
        .collect::<Vec<String>>()
        .join("\n")
}

async fn clear_channel(channel: &GuildChannel, ctx: &AppContext<'_>) -> anyhow::Result<()> {
    for msg in channel.messages(ctx, GetMessages::new()).await? {
        msg.delete(ctx).await?
    }

    Ok(())
}
