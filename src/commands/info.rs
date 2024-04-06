use std::collections::HashMap;

use anyhow::Context;
use chrono::{DateTime, Utc};
use poise::CreateReply;
use serde::ser::Error;
use serenity::all::{Builder, Member, PartialGuild, RichInvite, RoleId, User, UserId};
use serenity::builder::CreateInvite;
use serenity::futures::io::ReadToString;

use crate::error::respond_error;
use crate::util::builder::default_embed;
use crate::util::format::{display_time, escape_markdown, time};
use crate::Context as AppContext;

/// Get information.
#[poise::command(
    slash_command,
    guild_only = true,
    subcommands("server", "user", "members", "admins", "avatar"),
    subcommand_required
)]
pub async fn info(ctx: AppContext<'_>) -> anyhow::Result<()> {
    Ok(())
}

#[poise::command(slash_command, guild_only = true)]
async fn server(ctx: AppContext<'_>) -> anyhow::Result<()> {
    let Some(guild_id) = ctx.guild_id() else {
        anyhow::bail!("Failed to get guild id.");
    };

    let created_at = guild_id.created_at().to_utc();
    let partial_guild = guild_id.to_partial_guild_with_counts(&ctx).await?;

    let invite = match create_invite(&ctx).await {
        Ok(invite) => invite,
        Err(e) => {
            return respond_error("Failed to create invite", e, &ctx).await;
        }
    };

    let Some(member_count) = partial_guild.approximate_member_count else {
        ctx.say("Failed to get guild member count.").await?;
        return Ok(());
    };

    let embed = default_embed(&ctx.author())
        .title(format!("Server Info {}", partial_guild.name))
        .field("Membercount", member_count.to_string(), false)
        .field("Guild Created At", display_time(created_at), false)
        .field("Permanent Invite Link", invite.url(), false)
        .thumbnail(partial_guild.icon_url().unwrap_or_default());

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command)]
async fn user(
    ctx: AppContext<'_>,
    #[description = "Select a user."] target: User,
) -> anyhow::Result<()> {
    Ok(())
}

#[poise::command(slash_command)]
async fn members(ctx: AppContext<'_>) -> anyhow::Result<()> {
    let Some(guild_id) = ctx.guild_id() else {
        anyhow::bail!("Failed to get guild id.");
    };

    let partial_guild = guild_id.to_partial_guild(&ctx).await?;
    let members_role_id = ctx.data().config.roles.members;

    let member_names = get_member_names_per_role(&ctx, &partial_guild, &members_role_id).await?;

    let embed = default_embed(&ctx.author())
        .title("Info Members")
        .description(member_names.join("\n"))
        .field("Member Count", member_names.len().to_string(), false)
        .thumbnail(partial_guild.icon_url().unwrap_or_default());

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command)]
async fn admins(ctx: AppContext<'_>) -> anyhow::Result<()> {
    let Some(guild_id) = ctx.guild_id() else {
        anyhow::bail!("Failed to get guild id.");
    };

    let partial_guild = guild_id.to_partial_guild(&ctx).await?;
    let admin_role_id = ctx.data().config.roles.admin;

    let admin_names = get_member_names_per_role(&ctx, &partial_guild, &admin_role_id).await?;

    let embed = default_embed(&ctx.author())
        .title("Info Admins")
        .description(admin_names.join("\n"))
        .field("Admin Count", admin_names.len().to_string(), false)
        .thumbnail(partial_guild.icon_url().unwrap_or_default());

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command)]
async fn avatar(
    ctx: AppContext<'_>,
    #[description = "Select a user."] target: User,
) -> anyhow::Result<()> {
    Ok(())
}

async fn create_invite(ctx: &AppContext<'_>) -> anyhow::Result<RichInvite> {
    match ctx
        .data()
        .config
        .channels
        .invite
        .to_channel(ctx)
        .await?
        .guild()
    {
        Some(channel) => channel
            .create_invite(
                &ctx,
                CreateInvite::new().max_age(0).max_uses(0).unique(false),
            )
            .await
            .context("Failed to create invite"),
        None => {
            return Err(anyhow::anyhow!("Failed to get invite channel channel."));
        }
    }
}

async fn get_member_names_per_role(ctx: &AppContext<'_>, guild: &PartialGuild, role_id: &RoleId) -> anyhow::Result<Vec<String>> {
    let members = guild.members(&ctx, None, None).await?;

    let mut member_names = Vec::new();

    for member in members {
        if member.roles.contains(role_id) {
            member_names.push(member.user.global_name.unwrap_or(member.user.name));
        }
    }

    member_names.sort_unstable();

    Ok(member_names)
}
