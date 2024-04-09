use anyhow::Context;
use poise::CreateReply;
use pterodactyl_api::client::ServerState;
use serde::Deserialize;

use crate::{
    config::{Config, MinecraftConfig, ServerChoice},
    error::respond_error,
    util::{
        builder::default_embed,
        pterodactyl::{DisplayServerState, PteroClient},
        rcon::run_rcon_command,
    },
    Context as AppContext,
};

#[poise::command(slash_command, guild_only = true)]
pub async fn status(
    ctx: AppContext<'_>,
    #[description = "Choose a server to run the command on."] server_choice: ServerChoice,
) -> anyhow::Result<()> {
    ctx.defer().await?;

    let guild = ctx
        .partial_guild()
        .await
        .context("Failed to fetch the guild this interaction was created in")?;

    let server_config = ctx.data().config.minecraft.get(server_choice);

    let server_state =
        match PteroClient::server_state(&ctx.data().config.pterodactyl, server_config).await {
            Ok(state) => state,
            Err(e) => {
                return respond_error(
                    format!(
                        "Failed to get server state for {} from the pterodactyl API",
                        server_choice
                    ),
                    e,
                    &ctx,
                )
                .await;
            }
        };

    if server_state != ServerState::Running {
        ctx.say(format!(
            "{} is currently {}!",
            server_choice,
            DisplayServerState(server_state)
        ))
        .await?;
        return Ok(());
    }

    let mc_status = match mc_status(&ctx.data().config.minecraft, server_choice).await {
        Ok(status) => status,
        Err(e) => {
            return respond_error(
                format!("Failed to get status for {server_choice} from the mcstatus.io API"),
                e,
                &ctx,
            )
            .await;
        }
    };

    let server_metrics = match get_server_metrics(server_choice, &ctx.data().config).await {
        Ok(metrics) => metrics,
        Err(e) => {
            return respond_error(
                format!("Failed to get or calculate metrics for {server_choice}"),
                e,
                &ctx,
            )
            .await;
        }
    };

    let title = format!("{} {}", guild.name, server_choice);
    let colour = calculate_embed_color(server_metrics.performance.mspt);
    let icon_url = guild.icon_url().unwrap_or_default();

    let version = mc_status
        .version
        .map(|v| v.name_raw)
        .unwrap_or("Unknown version".to_string());

    let performance = format!(
        "**{}** MSPT | **{}** TPS",
        server_metrics.performance.mspt, server_metrics.performance.tps
    );

    let mobcaps = format!(
        "Overworld: {}\nThe Nether: {}\nThe End: {}",
        server_metrics.mobcap.overworld,
        server_metrics.mobcap.the_nether,
        server_metrics.mobcap.the_end
    );

    let player_count = format!(
        "online: **{}** | max: **{}**",
        server_metrics.players.count, server_metrics.players.max
    );

    let player_list = if server_metrics.players.count == 0 {
        "There is currently nobody online.".to_string()
    } else {
        server_metrics.players.playerlist.join("\n")
    };

    let embed = default_embed(ctx.author())
        .title(title)
        .colour(colour)
        .thumbnail(icon_url)
        .field("Status", "Online", false)
        .field("Version", version, false)
        .field("Performance", performance, false)
        .field("Hostile Mobcaps", mobcaps, false)
        .field("Playercount", player_count, false)
        .field("Playerlist", player_list, false);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct McStatusResponse {
    version: Option<McStatusVersionResponse>,
}

#[derive(Debug, Deserialize)]
struct McStatusVersionResponse {
    name_raw: String,
}

struct Mobcap {
    overworld: String,
    the_nether: String,
    the_end: String,
}

struct ServerPerformance {
    mspt: u32,
    tps: u32,
}

struct PlayerMetrics {
    count: u16,
    max: u16,
    playerlist: Vec<String>,
}

struct ServerMetrics {
    mobcap: Mobcap,
    performance: ServerPerformance,
    players: PlayerMetrics,
}

async fn mc_status(
    mc_config: &MinecraftConfig,
    server_choice: ServerChoice,
) -> anyhow::Result<McStatusResponse> {
    reqwest::get(format!(
        "https://api.mcstatus.io/v2/status/java/{}:{}",
        mc_config.get(server_choice).host,
        mc_config.get(server_choice).port
    ))
    .await?
    .json::<McStatusResponse>()
    .await
    .context("Failed to parse status response from mcstatus.io API")
}

async fn get_server_metrics(
    server: ServerChoice,
    config: &Config,
) -> anyhow::Result<ServerMetrics> {
    let commands = vec![
        "execute in minecraft:overworld run script run get_mob_counts('monster')".to_string(),
        "execute in minecraft:the_nether run script run get_mob_counts('monster')".to_string(),
        "execute in minecraft:the_end run script run get_mob_counts('monster')".to_string(),
        "script run reduce(system_info('server_last_tick_times'), _a+_, 0)/100".to_string(),
        "list".to_string(),
    ];

    let error_message = format!("Failed to execute the server metrics scripts on {server}");

    let responses = run_rcon_command(server, config, commands)
        .await
        .context(error_message.clone())?;

    if responses.len() != 5 {
        anyhow::bail!(error_message.clone())
    }

    let mut safe_responses = Vec::new();

    for r in responses {
        if let Some(r) = r {
            safe_responses.push(r)
        } else {
            anyhow::bail!(error_message);
        };
    }

    let mobcap_responses = safe_responses
        .clone()
        .into_iter()
        .take(3)
        .collect::<Vec<String>>();
    let performance_response = safe_responses[3].clone();
    let list_response = safe_responses[4].clone();

    let metrics = ServerMetrics {
        mobcap: parse_mobcaps(mobcap_responses)?,
        performance: parse_performance(performance_response)?,
        players: parse_playerlist(list_response)?,
    };

    Ok(metrics)
}

fn parse_performance(performance_response: String) -> anyhow::Result<ServerPerformance> {
    let split = performance_response.split(' ').collect::<Vec<&str>>();

    if split.len() != 4 {
        anyhow::bail!("Error parsing performance response: {performance_response}");
    }

    let mspt = split[2].to_string().parse::<f32>()?.round() as u32;

    if mspt == 0 {
        anyhow::bail!("MSPT response was 0");
    }

    let mut tps = 20;

    if mspt > 50 {
        tps = ((1000 / mspt) * 10) / 10;
    }

    Ok(ServerPerformance { mspt, tps })
}

fn parse_playerlist(list_response: String) -> anyhow::Result<PlayerMetrics> {
    let split = list_response.split(' ').collect::<Vec<&str>>();

    if split.len() < 8 {
        anyhow::bail!("Failed to parse list response: {list_response}");
    }

    let count = split[2].parse::<u16>()?;
    let max = split[7].parse::<u16>()?;

    let mut playerlist: Vec<String> = Vec::new();

    if count == 0 {
        return Ok(PlayerMetrics {
            count,
            max,
            playerlist,
        });
    }

    let first_split = list_response.split(": ").collect::<Vec<&str>>();

    if first_split.len() != 2 {
        anyhow::bail!("Failed to parse list response: {list_response}");
    }

    first_split[1]
        .split(", ")
        .map(|p| p.to_string())
        .for_each(|e| playerlist.push(e));

    Ok(PlayerMetrics {
        count,
        max,
        playerlist,
    })
}

fn parse_mobcaps(mobcap_responses: Vec<String>) -> anyhow::Result<Mobcap> {
    let r_one = regex::Regex::new(r"^.{0,3}|\(.*\)|\[\[\]\]")?;
    let r_two = regex::Regex::new(r", ")?;

    let replaced: Vec<String> = mobcap_responses
        .iter()
        .map(|res| {
            r_two
                .replace_all(&r_one.replace_all(res, ""), " | ")
                .to_string()
        })
        .collect();

    if replaced.len() != 3 {
        anyhow::bail!("Failed to parse mobcap responses using regex");
    }

    Ok(Mobcap {
        overworld: replaced[0].to_string(),
        the_nether: replaced[1].to_string(),
        the_end: replaced[2].to_string(),
    })
}

fn calculate_embed_color(mspt: u32) -> u32 {
    match mspt {
        30..=39 => 16_769_536,
        40..=49 => 16_737_843,
        50.. => 13_382_451,
        _ => 6_736_998,
    }
}
