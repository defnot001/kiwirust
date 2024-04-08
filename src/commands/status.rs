use anyhow::Context;
use poise::serenity_prelude as serenity;
use pterodactyl_api::client::ServerState;
use serde::Deserialize;

use crate::{
    config::{MinecraftConfig, ServerChoice, ServerConfig},
    error::respond_error,
    util::pterodactyl::{DisplayServerState, PteroClient},
    Context as AppContext,
};

#[poise::command(slash_command, guild_only = true)]
pub async fn status(
    ctx: AppContext<'_>,
    #[description = "Choose a server to run the command on."] server_choice: ServerChoice,
) -> anyhow::Result<()> {
    ctx.defer().await?;

    let server_config = ctx.data().config.minecraft.server_config(&server_choice);

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

    let mc_status = match mc_status(&ctx.data().config.minecraft, &server_choice).await {
        Ok(status) => status,
        Err(e) => {
            return respond_error(format!("Failed to get status for {server_choice} from the mcstatus.io API"), e, &ctx).await;
        }
    };

    todo!()
}

#[derive(Debug, Deserialize)]
struct McStatusResponse {
    version: Option<McStatusVersionResponse>,
    players: Option<McStatusPlayersResponse>,
}

#[derive(Debug, Deserialize)]
struct McStatusVersionResponse {
    name_raw: String,
}

#[derive(Debug, Deserialize)]
struct McStatusPlayersResponse {
    max: u16,
}

async fn mc_status(
    mc_config: &MinecraftConfig,
    server_choice: &ServerChoice,
) -> anyhow::Result<McStatusResponse> {
    reqwest::get(format!(
        "https://api.mcstatus.io/v2/status/java/{}:{}",
        mc_config.host.to_string(),
        mc_config.server_config(server_choice).port
    ))
    .await?
    .json::<McStatusResponse>()
    .await
    .context("Failed to parse status response from mcstatus.io API")
}
