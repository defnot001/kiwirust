use std::str::FromStr;

use anyhow::Context;
use chrono::{offset::LocalResult, DateTime, TimeZone, Utc};
use poise::serenity_prelude as serenity;
use poise::CreateReply;
use pterodactyl_api::client::backups::Backup;
use serenity::{
    Builder, ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton,
    CreateInteractionResponse, CreateInteractionResponseMessage, EditInteractionResponse,
    PartialGuild,
};
use uuid::Uuid;

use crate::config::PterodactylConfig;
use crate::config::ServerConfig;
use crate::util::random_utils::confirm_cancel_component;
use crate::{
    config::ServerChoice,
    error::respond_error,
    util::{
        builder::default_embed,
        format::{display_bytes, inline_code, time, TimestampStyle},
        pterodactyl::PteroClient,
        random_utils::maybe_set_guild_thumbnail,
    },
    Context as AppContext,
};

/// Control backups on a minecraft server.
#[poise::command(
    slash_command,
    guild_only = true,
    subcommands("list", "details", "create", "delete"),
    subcommand_required,
    track_edits
)]
pub async fn backup(_: AppContext<'_>) -> anyhow::Result<()> {
    Ok(())
}

/// List all backups from a minecraft server.
#[poise::command(slash_command, guild_only = true)]
async fn list(
    ctx: AppContext<'_>,
    #[description = "Choose a server."] server_choice: ServerChoice,
) -> anyhow::Result<()> {
    ctx.defer().await?;

    let guild = ctx
        .partial_guild()
        .await
        .context("Failed to fetch the guild this interaction was created in")?;

    let backups = match PteroClient::backup_list(
        &ctx.data().config.pterodactyl,
        ctx.data().config.minecraft.get(server_choice),
    )
    .await
    {
        Ok(b) => b,
        Err(e) => {
            return respond_error(
                format!("Failed to get backup list from {server_choice}"),
                e,
                &ctx,
            )
            .await;
        }
    };

    if backups.is_empty() {
        ctx.say(format!("There are currently no backups on {server_choice}"))
            .await?;
        return Ok(());
    }

    let display_backups = backups
        .into_iter()
        .map(|b| {
            let created_at = match Utc.timestamp_opt(b.created_at.unix_timestamp(), 0) {
                LocalResult::Single(date_time) => date_time,
                LocalResult::Ambiguous(earliest, ..) => earliest,
                LocalResult::None => {
                    anyhow::bail!("Failed to parse the date of backup creation!");
                }
            };

            let formatted_time = time(created_at, TimestampStyle::ShortDateTime);

            Ok(format!(
                "**Time**: {formatted_time}\n**Name**: {}\n**UUID**: {}",
                inline_code(b.name),
                inline_code(b.uuid)
            ))
        })
        .take(20)
        .collect::<Vec<anyhow::Result<String>>>();

    if !display_backups.iter().all(|b| b.is_ok()) {
        ctx.say(format!(
            "Failed to parse creation date for backups from {server_choice}!"
        ))
        .await?;
        return Ok(());
    }

    let display_backups = display_backups
        .into_iter()
        .filter_map(|b| b.ok())
        .collect::<Vec<String>>()
        .join("\n\n");

    let embed = default_embed(ctx.author())
        .title(format!("Backup List for {} {server_choice}", guild.name))
        .description(display_backups);

    let embed = maybe_set_guild_thumbnail(embed, &guild);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// Get details about a backup.
#[poise::command(slash_command, guild_only = true)]
async fn details(
    ctx: AppContext<'_>,
    #[description = "Choose a server."] server_choice: ServerChoice,
    #[description = "The ID of the backup you want to get the details of. You can get the ID from the list subcommand."]
    backup_id: String,
) -> anyhow::Result<()> {
    let guild = ctx
        .partial_guild()
        .await
        .context("Failed to fetch the guild this interaction was created in")?;

    let uuid = match Uuid::from_str(backup_id.as_str()) {
        Ok(uuid) => uuid,
        Err(e) => {
            return respond_error("Failed to parse UUID", e, &ctx).await;
        }
    };

    let backup = match PteroClient::backup_details(
        &ctx.data().config.pterodactyl,
        ctx.data().config.minecraft.get(server_choice),
        uuid,
    )
    .await
    {
        Ok(backup) => backup,
        Err(e) => {
            return respond_error(
                format!("Failed to get backup with uuid {uuid} from {server_choice}"),
                e,
                &ctx,
            )
            .await;
        }
    };

    let completed = if backup.completed_at.is_some() {
        "true"
    } else {
        "false"
    };

    let locked = if backup.is_locked { "true" } else { "false" };

    let created_at = match Utc.timestamp_opt(backup.created_at.unix_timestamp(), 0) {
        LocalResult::Single(date_time) => time(date_time, TimestampStyle::ShortDateTime),
        LocalResult::Ambiguous(earliest, ..) => time(earliest, TimestampStyle::ShortDateTime),
        LocalResult::None => {
            anyhow::bail!("Failed to parse the date of backup creation!");
        }
    };

    let completed_at = if let Some(completed) = backup.completed_at {
        match Utc.timestamp_opt(backup.created_at.unix_timestamp(), 0) {
            LocalResult::Single(date_time) => time(date_time, TimestampStyle::ShortDateTime),
            LocalResult::Ambiguous(earliest, ..) => time(earliest, TimestampStyle::ShortDateTime),
            LocalResult::None => {
                anyhow::bail!("Failed to parse the date of backup creation!");
            }
        }
    } else {
        "Backup not completed".to_string()
    };

    let embed = default_embed(ctx.author())
        .title(format!("Backup details for {} {server_choice}", guild.name))
        .field("Name", backup.name, false)
        .field("UUID", inline_code(backup.uuid), false)
        .field("Size", display_bytes(backup.bytes), false)
        .field("Successful", completed, false)
        .field("Locked", locked, false)
        .field("Created At", created_at, false)
        .field("Completed At", completed_at, false);

    let embed = maybe_set_guild_thumbnail(embed, &guild);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// Create a backup on a minecraft server.
#[poise::command(slash_command, guild_only = true, track_edits)]
async fn create(
    ctx: AppContext<'_>,
    #[description = "Choose a server."] server_choice: ServerChoice,
    #[description = "The name of the backup."] backup_name: Option<String>,
    #[description = "Wether the backup should be locked."] locked: Option<bool>,
) -> anyhow::Result<()> {
    let guild = ctx
        .partial_guild()
        .await
        .context("Failed to fetch the guild this interaction was created in")?;

    let server_config = ctx.data().config.minecraft.get(server_choice);

    if server_config.backup_limit == 0 {
        ctx.say(format!(
            "{} {server_choice} does not allow backups",
            guild.name
        ));
        return Ok(());
    }

    let backups = PteroClient::backup_list(&ctx.data().config.pterodactyl, server_config).await?;

    if backups.len() == server_config.backup_limit as usize {
        if !has_unlocked_backups(&backups) {
            ctx.say(format!(
                "{} {server_choice} does not have any unlocked backups to replace!",
                guild.name
            ));
            return Ok(());
        }

        handle_replace_oldest_backup(ctx, &guild, server_choice, backups, backup_name, locked)
            .await?;
        return Ok(());
    }

    let handle = ctx
        .say(format!(
            "Creating backup for {} {server_choice}...",
            guild.name
        ))
        .await?;

    let created = PteroClient::create_backup_and_wait(
        &ctx.data().config.pterodactyl,
        server_config,
        backup_name,
        locked,
        ctx.author(),
    )
    .await?;

    let mut content = String::new();

    if created.completed_at.is_some() {
        let duration = calculate_completion_seconds(&created).unwrap();
        content = format!(
            "Successfully created backup on {} {server_choice}. This took {} seconds!",
            guild.name, duration
        );
    } else {
        content = format!(
            "Successfully created backup on {} {server_choice} but failed to wait for completion!",
            guild.name
        );
    }

    handle
        .edit(ctx, CreateReply::default().content(content))
        .await?;

    Ok(())
}

/// Delete a backup from a minecraft server.
#[poise::command(slash_command, guild_only = true)]
async fn delete(
    ctx: AppContext<'_>,
    #[description = "Choose a server."] server_choice: ServerChoice,
    #[description = "The ID of the backup you want to delete. You can get the ID from the list subcommand."]
    backup_id: String,
) -> anyhow::Result<()> {
    ctx.defer().await?;

    let guild = ctx
        .partial_guild()
        .await
        .context("Failed to fetch the guild this interaction was created in")?;

    let uuid = match Uuid::from_str(backup_id.as_str()) {
        Ok(uuid) => uuid,
        Err(e) => {
            return respond_error("Failed to parse UUID", e, &ctx).await;
        }
    };

    let mut response = String::new();

    match PteroClient::delete_backup(
        &ctx.data().config.pterodactyl,
        ctx.data().config.minecraft.get(server_choice),
        uuid,
    )
    .await
    {
        Ok(_) => {
            ctx.say(format!(
                "Successfully deleted backup from {} {server_choice}.",
                guild.name
            ))
            .await?;

            Ok(())
        }
        Err(e) => {
            respond_error(
                format!(
                    "Failed to delete backup {} from {} {server_choice}",
                    inline_code(uuid),
                    guild.name
                ),
                e,
                &ctx,
            )
            .await
        }
    }
}

async fn handle_replace_oldest_backup(
    ctx: AppContext<'_>,
    guild: &PartialGuild,
    server_choice: ServerChoice,
    backup_list: Vec<Backup>,
    backup_name: Option<String>,
    locked: Option<bool>,
) -> anyhow::Result<()> {
    let interaction_id = ctx.id();

    let buttons = confirm_cancel_component();
    let reply = CreateReply::default().components(buttons).content(format!(
        "This command will delete the oldest backup for {} {server_choice} because the backup limit is reached for this server. Are you sure you want to continue? This cannot be undone!",
        guild.name
    ));

    ctx.send(reply).await?;

    while let Some(collector) = ComponentInteractionCollector::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .await
        .filter(move |c| {
            c.data.custom_id.as_str() == "confirm" || c.data.custom_id.as_str() == "cancel"
        })
    {
        let custom_id = collector.data.custom_id.clone();

        if custom_id.as_str() == "cancel" {
            let response = CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(format!(
                        "Cancelled creating a backup for {} {server_choice}!",
                        guild.name
                    ))
                    .components(vec![]),
            );

            collector.create_response(&ctx, response).await?;
            return Ok(());
        }

        if custom_id.as_str() == "confirm" {
            if let Err(e) = delete_oldest_non_locked_backup(
                &ctx.data().config.pterodactyl,
                ctx.data().config.minecraft.get(server_choice),
                &backup_list,
            )
            .await
            {
                let response = CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .content(format!(
                            "Failed to delete the oldest backup from {} {server_choice}!",
                            guild.name
                        ))
                        .components(vec![]),
                );

                collector.create_response(&ctx, response).await?;
                return Ok(());
            }

            let response = CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(format!(
                        "Creating backup for {} {server_choice}...",
                        guild.name
                    ))
                    .components(vec![]),
            );

            collector.create_response(&ctx, response).await?;

            let created = match PteroClient::create_backup_and_wait(
                &ctx.data().config.pterodactyl,
                ctx.data().config.minecraft.get(server_choice),
                backup_name.clone(),
                locked,
                ctx.author(),
            )
            .await
            {
                Ok(b) => b,
                Err(e) => {
                    tracing::error!(
                        "Failed to create backup on {} {}: {e}",
                        guild.name,
                        server_choice
                    );

                    collector
                        .edit_response(
                            &ctx,
                            EditInteractionResponse::new().content(format!(
                                "Failed to create a new backup for {} {server_choice}!",
                                guild.name
                            )),
                        )
                        .await?;

                    return Ok(());
                }
            };

            let mut content = String::new();

            if created.completed_at.is_some() {
                let duration = calculate_completion_seconds(&created).unwrap();
                content = format!(
                    "Successfully deleted the oldest backup and created a new one on {} {server_choice}. This took {} seconds!",
                    guild.name, duration
                );
            } else {
                content = format!(
                    "Successfully deleted the oldest backup and created a new one on {} {server_choice} but failed to wait for completion!",
                    guild.name
                );
            }

            collector
                .edit_response(&ctx, EditInteractionResponse::new().content(content))
                .await?;

            return Ok(());
        }
    }

    Ok(())
}

async fn delete_oldest_non_locked_backup(
    ptero_config: &PterodactylConfig,
    server_config: &ServerConfig,
    backups: &[Backup],
) -> anyhow::Result<()> {
    let server_choice = ServerChoice::try_from(server_config)?;

    let mut backups = backups
        .iter()
        .filter(|&b| !b.is_locked)
        .collect::<Vec<&Backup>>();

    if backups.is_empty() {
        anyhow::bail!("There are no non-locked backups on {server_choice}")
    }

    backups.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    let first = backups.first().context("There is no backup to delete")?;

    PteroClient::delete_backup(ptero_config, server_config, first.uuid).await?;

    Ok(())
}

fn calculate_completion_seconds(backup: &Backup) -> Option<i64> {
    let created_at = backup.created_at;
    let completed_at = backup.completed_at;

    if let Some(completed) = completed_at {
        let duration = completed - created_at;
        Some(duration.whole_seconds())
    } else {
        None
    }
}

fn has_unlocked_backups(backups: &[Backup]) -> bool {
    backups.iter().any(|b| !b.is_locked)
}
