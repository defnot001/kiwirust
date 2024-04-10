use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};

use anyhow::Context;
use poise::{serenity_prelude as serenity, CreateReply};
use rcon::Builder;
use serenity::{CreateEmbed, User};

use crate::{
    config::{Config, MinecraftConfig, ServerChoice, ServerConfig},
    error::respond_error,
    util::{
        builder::default_embed, format::escape_markdown, random_utils::sort_player_list,
        rcon::run_rcon_command,
    },
    Context as AppContext,
};

/// Get information about the whitelist & add/remove users.
#[poise::command(
    slash_command,
    guild_only = true,
    subcommands("add", "remove", "list"),
    subcommand_required
)]
pub async fn whitelist(_: AppContext<'_>) -> anyhow::Result<()> {
    Ok(())
}

/// Add a player to the whitelist on all servers.
#[poise::command(slash_command, guild_only = true)]
async fn add(
    ctx: AppContext<'_>,
    #[description = "The IGN of the player to add."] ign: String,
) -> anyhow::Result<()> {
    ctx.defer().await;

    if ign.trim().is_empty() {
        ctx.say("IGN cannot be empty!");
        return Ok(());
    }

    let embed =
        match add_remove_whitelist(ign.as_str(), ctx.data().config.minecraft.clone(), true).await {
            Ok(r) => build_whitelist_report_embed(ctx.author(), ign, &r),
            Err(e) => {
                return respond_error(
                    format!("Something went wrong trying to add {ign} to the whitelist!"),
                    e,
                    &ctx,
                )
                .await;
            }
        };

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// Remove a player from the whitelist on all servers.
#[poise::command(slash_command, guild_only = true)]
async fn remove(
    ctx: AppContext<'_>,
    #[description = "The IGN of the player to remove."] ign: String,
) -> anyhow::Result<()> {
    ctx.defer().await;

    if ign.trim().is_empty() {
        ctx.say("IGN cannot be empty!");
        return Ok(());
    }

    let embed = match add_remove_whitelist(ign.as_str(), ctx.data().config.minecraft.clone(), false)
        .await
    {
        Ok(r) => build_whitelist_report_embed(ctx.author(), ign, &r),
        Err(e) => {
            return respond_error(
                format!("Something went wrong trying to remove {ign} from the whitelist!"),
                e,
                &ctx,
            )
            .await;
        }
    };

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command, guild_only = true)]
async fn list(
    ctx: AppContext<'_>,
    #[description = "The server to get the whitelist from."] server: ServerChoice,
) -> anyhow::Result<()> {
    ctx.defer().await?;

    let whitelist = match get_whitelist(ctx.data().config.minecraft.get(server)).await {
        Ok(whitelist) => whitelist,
        Err(e) => {
            return respond_error(
                format!("Failed to get whitelist list response from {server}"),
                e,
                &ctx,
            )
            .await;
        }
    };

    if whitelist.is_empty() {
        ctx.say(format!("There are no whitelist players on {server}!"))
            .await?;
        return Ok(());
    }

    let count = whitelist.len();
    let display = whitelist
        .into_iter()
        .map(escape_markdown)
        .collect::<Vec<String>>()
        .join("\n");

    let embed = default_embed(ctx.author())
        .title(format!("{server} Whitelist"))
        .description(display)
        .field("Count", count.to_string(), false);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

async fn get_whitelist(server_config: &ServerConfig) -> anyhow::Result<Vec<String>> {
    let server_choice = ServerChoice::try_from(server_config)?;

    let mut player_list = run_rcon_command(server_config, vec!["whitelist list"])
        .await?
        .into_iter()
        .flatten()
        .next()
        .context("{server_choice} returned an unexpected or empty response")?
        .split(": ")
        .nth(1)
        .context("{server_choice} returned an unexpected response")?
        .split(", ")
        .map(str::to_string)
        .collect::<Vec<String>>();

    sort_player_list(&mut player_list);

    Ok(player_list)
}

enum WhitelistResult {
    Success,
    Already,
    Fail,
}

impl FromStr for WhitelistResult {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        if s == "Player is already whitelisted" || s == "Player is not whitelisted" {
            Ok(Self::Already)
        } else if s.starts_with("Added") || s.starts_with("Removed") {
            Ok(Self::Success)
        } else {
            anyhow::bail!("Failed to parse op response")
        }
    }
}

enum OpResult {
    Success,
    Already,
    Fail,
}

impl FromStr for OpResult {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        if s.starts_with("Nothing changed.") {
            Ok(Self::Already)
        } else if s.starts_with("Made") {
            Ok(Self::Success)
        } else {
            anyhow::bail!("Failed to parse op response")
        }
    }
}

struct WhitelistResultElement {
    server: ServerChoice,
    whitelist: WhitelistResult,
    op: Option<OpResult>,
}

fn get_whitelist_add_commands(ign: &str, operator: bool) -> Vec<String> {
    let mut commands = vec![format!("whitelist add {ign}")];

    if operator {
        commands.push(format!("op {ign}"));
    }

    commands
}
fn get_whitelist_remove_commands(ign: &str, operator: bool) -> Vec<String> {
    let mut commands = vec![format!("whitelist remove {ign}")];

    if operator {
        commands.push(format!("deop {ign}"));
    }

    commands
}

async fn add_remove_whitelist(
    ign: impl Into<String>,
    configs: MinecraftConfig,
    add: bool,
) -> anyhow::Result<Vec<WhitelistResultElement>> {
    let ign = ign.into();

    let mut results = Vec::new();

    for config in configs {
        let server_choice = ServerChoice::try_from(&config)?;

        let commands = if add {
            get_whitelist_add_commands(ign.as_str(), config.operator)
        } else {
            get_whitelist_remove_commands(ign.as_str(), config.operator)
        };

        let (w, o) = run_whitelist_rcon(&config, commands).await?;

        let element = WhitelistResultElement {
            server: server_choice,
            whitelist: w,
            op: o,
        };

        results.push(element);
    }

    Ok(results)
}

async fn run_whitelist_rcon(
    config: &ServerConfig,
    commands: Vec<String>,
) -> anyhow::Result<(WhitelistResult, Option<OpResult>)> {
    let host = config.host.as_str().parse::<Ipv4Addr>()?;
    let addr = SocketAddr::new(host.into(), config.rcon_port);

    let mut connection = Builder::new()
        .enable_minecraft_quirks(true)
        .connect(addr, &config.rcon_password)
        .await?;

    let mut whitelist_result = WhitelistResult::Fail;
    let mut op_result = if commands.len() == 2 {
        Some(OpResult::Fail)
    } else {
        None
    };

    for (i, command) in commands.into_iter().enumerate() {
        let response = connection.cmd(command.as_str()).await;

        if let Ok(response) = response {
            if response.is_empty() {
                continue;
            }

            if i == 0 {
                whitelist_result = WhitelistResult::from_str(response.as_str())?;
            } else if i == 1 {
                op_result = Some(OpResult::from_str(response.as_str())?);
            } else {
                anyhow::bail!("Received unexpected response length from server");
            }
        } else {
            continue;
        }
    }

    Ok((whitelist_result, op_result))
}

fn build_whitelist_report_embed(
    interaction_user: &User,
    ign: String,
    elements: &[WhitelistResultElement],
) -> CreateEmbed {
    let fields = elements.iter().map(|element| {
        let name = element.server.to_string();
        let mut value = match element.whitelist {
            WhitelistResult::Success => "**Whitelist**: Success".to_string(),
            WhitelistResult::Already => "**Whitelist**: Already".to_string(),
            WhitelistResult::Fail => "**Whitelist**: Failed".to_string(),
        };

        if element.op.is_some() {
            let op = match element.op.as_ref().unwrap() {
                OpResult::Success => "\n**Operator**: Success",
                OpResult::Already => "\n**Operator**: Already",
                OpResult::Fail => "\n**Operator**: Failed",
            };

            value.push_str(op);
        }

        (name, value, false)
    });

    default_embed(interaction_user)
        .title(format!("Whitelist Results for {}", ign))
        .fields(fields)
}
