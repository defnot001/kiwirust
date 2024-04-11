use anyhow::Context;
use chrono::{offset::LocalResult, DateTime, TimeZone, Utc};
use poise::{serenity_prelude as serenity, CreateReply};

use serenity::User;
use uuid::Uuid;

use crate::{
    database::model::member::{CreateMember, MemberModelController, UpdateMember},
    error::respond_error,
    util::{
        builder::default_embed,
        format::{display, display_time, escape_markdown, fdisplay, inline_code},
        mojang::MojangAPI,
        random_utils::{maybe_set_guild_thumbnail, sort_player_list},
    },
    Context as AppContext,
};
/// Command to interact with Members in the database.
#[poise::command(
    slash_command,
    guild_only = true,
    subcommands("list", "info", "add", "update", "remove"),
    subcommand_required
)]
pub async fn member(_: AppContext<'_>) -> anyhow::Result<()> {
    Ok(())
}

#[poise::command(slash_command, guild_only = true)]
async fn list(ctx: AppContext<'_>) -> anyhow::Result<()> {
    let guild = ctx
        .partial_guild()
        .await
        .context("Failed to fetch the guild this interaction was created in")?;

    let members = match MemberModelController::get_all(&ctx.data().db_pool).await {
        Ok(members) => members,
        Err(e) => {
            return respond_error("Failed to get members from the database", e, &ctx).await;
        }
    };

    if members.is_empty() {
        ctx.say("There are no members in the database.").await?;
        return Ok(());
    }

    let mut member_names = Vec::with_capacity(members.len());

    for member in members {
        let member_user = member.discord_id.to_user(&ctx).await?;
        let username = member_user.global_name.unwrap_or(member_user.name);

        member_names.push(format!(
            "{} ({})",
            escape_markdown(username),
            inline_code(member_user.id.to_string())
        ));
    }

    sort_player_list(&mut member_names);

    let embed = default_embed(ctx.author())
        .title(format!("Memberlist for {}", guild.name))
        .description(member_names.join("\n"));

    let embed = maybe_set_guild_thumbnail(embed, &guild);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command, guild_only = true)]
async fn info(
    ctx: AppContext<'_>,
    #[description = "The Member to display information about."] user: User,
) -> anyhow::Result<()> {
    ctx.defer().await?;

    let Some(mc_member) = MemberModelController::get_by_id(&ctx.data().db_pool, &user.id).await?
    else {
        ctx.say(format!(
            "{} is not a member in the database.",
            fdisplay(&user)
        ))
        .await?;
        return Ok(());
    };

    let mut profiles = Vec::new();

    for uuid in &mc_member.minecraft_uuids {
        profiles.push(MojangAPI::get_profile_from_uuid(uuid).await?)
    }

    if profiles.is_empty() {
        ctx.say(format!(
            "User {} does not have any minecraft accounts connected.",
            fdisplay(&user)
        ))
        .await?;
        return Ok(());
    }

    let skin_url = format!("https://visage.surgeplay.com/face/256/{}", profiles[0].id);

    let display_profiles = profiles
        .into_iter()
        .map(|p| {
            format!(
                "{} ({})",
                escape_markdown(p.name),
                inline_code(p.id.to_string())
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let display_trial = if mc_member.trial_member {
        "Yes".to_string()
    } else {
        "No".to_string()
    };

    let embed = default_embed(&user)
        .title(format!(
            "Member Info for {}",
            user.global_name.unwrap_or(user.name)
        ))
        .thumbnail(skin_url)
        .field("Discord ID", inline_code(user.id.to_string()), false)
        .field("Minecraft Usernames", display_profiles, false)
        .field("Member Since", display_time(mc_member.member_since), false)
        .field("Last Updated At", display_time(mc_member.updated_at), false)
        .field("Trial Member", display_trial, false);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command, guild_only = true)]
async fn add(
    ctx: AppContext<'_>,
    #[description = "The Member to add."] user: User,
    #[description = "The Member's In-Game Name(s). Separate multiple names with a comma (,)."]
    igns: String,
    #[description = "Wether the member is a trial Member."] trial_member: bool,
    #[description = "The date the Member joined the server. Format: YYYY-MM-DD"]
    member_since: Option<String>,
) -> anyhow::Result<()> {
    ctx.defer().await?;

    let member_since = if let Some(member_since_str) = member_since {
        match parse_date_string(member_since_str) {
            Ok(m) => m,
            Err(e) => {
                return respond_error("Failed to parse date", e, &ctx).await;
            }
        }
    } else {
        Utc::now()
    };

    let igns = igns
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();

    if igns.is_empty() {
        ctx.say("Ign's cannot be empty!").await?;
        return Ok(());
    }

    let profiles = match MojangAPI::get_profiles(igns).await {
        Ok(profiles) => profiles,
        Err(e) => {
            return respond_error("Failed to get profiles from the Mojang API", e, &ctx).await;
        }
    };

    let create_member = CreateMember {
        discord_id: user.id,
        trial_member,
        minecraft_uuids: profiles.into_iter().map(|p| p.id).collect::<Vec<Uuid>>(),
        member_since: member_since.naive_utc(),
    };

    match MemberModelController::create(&ctx.data().db_pool, create_member).await {
        Ok(_) => {
            ctx.say(format!(
                "Successfully added {} to the Memberlist.",
                fdisplay(&user)
            ))
            .await?;
            Ok(())
        }
        Err(e) => {
            if e.to_string() == "Unique constraint violation" {
                respond_error(format!("{} is already a member", display(&user)), e, &ctx).await
            } else {
                respond_error(
                    format!("Failed to add {} to the Memberlist", display(&user)),
                    e,
                    &ctx,
                )
                .await
            }
        }
    }
}

#[poise::command(slash_command, guild_only = true)]
async fn update(
    ctx: AppContext<'_>,
    #[description = "The Member to update."] user: User,
    #[description = "The Member's In-Game Name(s). Separate multiple names with a comma (,)."]
    igns: Option<String>,
    #[description = "Wether the member is a trial Member."] trial_member: Option<bool>,
    #[description = "The date the Member joined the server. Format: YYYY-MM-DD"]
    member_since: Option<String>,
) -> anyhow::Result<()> {
    ctx.defer().await?;

    let minecraft_uuids = if let Some(igns) = igns {
        let igns = igns
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();

        if igns.is_empty() {
            ctx.say("Ign's cannot be empty!").await?;
            return Ok(());
        }

        let uuids = match MojangAPI::get_profiles(igns).await {
            Ok(profiles) => profiles.into_iter().map(|p| p.id).collect::<Vec<Uuid>>(),
            Err(e) => {
                return respond_error("Failed to get profiles from the Mojang API", e, &ctx).await
            }
        };

        Some(uuids)
    } else {
        None
    };

    let member_since = if let Some(member_since) = member_since {
        match parse_date_string(member_since) {
            Ok(m) => Some(m.naive_utc()),
            Err(e) => {
                return respond_error("Failed to parse date", e, &ctx).await;
            }
        }
    } else {
        None
    };

    let update_member = UpdateMember {
        discord_id: user.id,
        trial_member,
        minecraft_uuids,
        member_since,
    };

    match MemberModelController::update(&ctx.data().db_pool, update_member).await {
        Ok(_) => {
            ctx.say(format!(
                "Successfully updated {} in the database.",
                fdisplay(&user)
            ))
            .await?;
            Ok(())
        }
        Err(e) => {
            respond_error(
                format!("Failed to update {} in the database", display(&user)),
                e,
                &ctx,
            )
            .await
        }
    }
}

#[poise::command(slash_command, guild_only = true)]
async fn remove(
    ctx: AppContext<'_>,
    #[description = "The Member to remove."] user: User,
) -> anyhow::Result<()> {
    ctx.defer().await?;

    match MemberModelController::delete(&ctx.data().db_pool, &user.id).await {
        Ok(_) => {
            ctx.say(format!(
                "Successfully removed {} from the Memberlist.",
                fdisplay(&user)
            ))
            .await?;

            Ok(())
        }
        Err(e) => {
            respond_error(
                format!("Failed to remove {} from the database.", display(&user)),
                e,
                &ctx,
            )
            .await
        }
    }
}

fn parse_date_string(date_str: String) -> anyhow::Result<DateTime<Utc>> {
    let split = date_str
        .split('-')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    if split.len() != 3 {
        anyhow::bail!(
            "Error parsing date string. Expected len = 3, got {}",
            split.len()
        )
    }

    let year = split[0].parse::<i32>()?;
    let month = split[1].parse::<u32>()?;
    let day = split[2].parse::<u32>()?;

    let LocalResult::Single(date) = Utc.with_ymd_and_hms(year, month, day, 12, 0, 0) else {
        anyhow::bail!("Failed to create Date from string.")
    };

    Ok(date)
}
