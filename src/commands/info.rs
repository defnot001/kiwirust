use anyhow::Context;
use poise::CreateReply;
use serenity::all::{Member, PartialGuild, RichInvite, RoleId, User};
use serenity::builder::{CreateAttachment, CreateInvite};
use url::Url;

use crate::error::respond_error;
use crate::util::builder::default_embed;
use crate::util::format::display_time;
use crate::util::random_utils::{maybe_set_guild_thumbnail, sort_player_list};
use crate::Context as AppContext;

/// Get information.
#[poise::command(
    slash_command,
    guild_only = true,
    subcommands("server", "user", "members", "admins", "avatar"),
    subcommand_required
)]
pub async fn info(_: AppContext<'_>) -> anyhow::Result<()> {
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

    let embed = default_embed(ctx.author())
        .title(format!("Server Info {}", partial_guild.name))
        .field("Membercount", member_count.to_string(), false)
        .field("Guild Created At", display_time(created_at), false)
        .field("Permanent Invite Link", invite.url(), false);

    let embed = maybe_set_guild_thumbnail(embed, &partial_guild);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command)]
async fn user(
    ctx: AppContext<'_>,
    #[description = "Select a user."] target: User,
) -> anyhow::Result<()> {
    let Some(guild) = ctx.partial_guild().await else {
        anyhow::bail!("Cannot find the guild this interaction was created in")
    };

    let name = target
        .global_name
        .as_deref()
        .unwrap_or(target.name.as_str());
    let avatar_url = target.avatar_url().unwrap_or(target.default_avatar_url());
    let joined_discord = display_time(target.created_at().to_utc());

    let embed = default_embed(ctx.author())
        .title(format!("User Info {name}"))
        .thumbnail(avatar_url)
        .field("Username", target.name.as_str(), false)
        .field("User ID", target.id.to_string(), false)
        .field("Joined Discord", joined_discord, false);

    let embed = if let Ok(member) = guild.member(&ctx, &target.id).await {
        let embed = if let Some(joined) = member.joined_at {
            embed.field(
                format!("Joined {}", guild.name),
                display_time(joined.to_utc()),
                false,
            )
        } else {
            embed
        };

        let roles = display_member_roles(&member, &guild, &ctx).await;

        embed.field("Roles", roles, false)
    } else {
        embed
    };

    ctx.send(CreateReply::default().embed(embed)).await?;

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

    let embed = default_embed(ctx.author())
        .title("Info Members")
        .description(member_names.join("\n"))
        .field("Member Count", member_names.len().to_string(), false);

    let embed = maybe_set_guild_thumbnail(embed, &partial_guild);

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

    let embed = default_embed(ctx.author())
        .title("Info Admins")
        .description(admin_names.join("\n"))
        .field("Admin Count", admin_names.len().to_string(), false);

    let embed = maybe_set_guild_thumbnail(embed, &partial_guild);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command)]
async fn avatar(
    ctx: AppContext<'_>,
    #[description = "Select a user."] target: User,
) -> anyhow::Result<()> {
    let mut avatar_url = Url::parse(
        target
            .avatar_url()
            .unwrap_or(target.default_avatar_url())
            .as_str(),
    )?;

    if avatar_url.query().is_none() {
        anyhow::bail!(
            "Failed to parse url size query param in url: {}",
            avatar_url.as_str()
        );
    };

    avatar_url.set_query(Some("size=4096"));

    let attachment = CreateAttachment::url(&ctx, avatar_url.as_str()).await?;

    ctx.send(CreateReply::default().attachment(attachment))
        .await
        .context("Failed to send message")?;

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
        None => Err(anyhow::anyhow!("Failed to get invite channel channel.")),
    }
}

async fn get_member_names_per_role(
    ctx: &AppContext<'_>,
    guild: &PartialGuild,
    role_id: &RoleId,
) -> anyhow::Result<Vec<String>> {
    let members = guild.members(&ctx, None, None).await?;

    let mut member_names = Vec::new();

    for member in members {
        if member.roles.contains(role_id) {
            member_names.push(member.user.global_name.unwrap_or(member.user.name));
        }
    }

    sort_player_list(&mut member_names);

    Ok(member_names)
}

async fn display_member_roles(
    member: &Member,
    guild: &PartialGuild,
    ctx: &AppContext<'_>,
) -> String {
    let Some(roles) = member.roles(ctx) else {
        return "None".to_string();
    };

    if roles.is_empty() {
        return "None".to_string();
    }

    // filter out the @everyone role
    let mut roles = roles
        .into_iter()
        .filter(|role| role.id != RoleId::from(guild.id.get()))
        .collect::<Vec<_>>();

    roles.sort_unstable();
    roles.reverse();

    roles
        .iter()
        .map(|role| format!("<@&{}>", role.id))
        .collect::<Vec<_>>()
        .join(", ")
}
