use ::serenity::all::Member;
use anyhow::Context;
use poise::serenity_prelude as serenity;

use crate::{
    config::ServerChoice,
    util::{format::fdisplay, rcon::run_rcon_command},
    Context as AppContext,
};

/// Get random pictures of animals.
#[poise::command(slash_command, guild_only = true)]
pub async fn run(
    ctx: AppContext<'_>,
    #[description = "Choose a server to run the command on."] server_choice: ServerChoice,
    #[description = "The command to run on the server."] command: String,
) -> anyhow::Result<()> {
    ctx.defer().await?;

    if let ServerChoice::Smp = server_choice {
        if !is_interaction_from_admin(&ctx).await? {
            return Err(anyhow::anyhow!(
                "You must be an admin to run arbitrary commands on SMP!"
            ));
        }
    }

    if command.is_empty() {
        return Err(anyhow::anyhow!("Command string cannot be empty."));
    }

    let response = run_rcon_command(&server_choice, &ctx.data().config, command)
        .await?
        .unwrap_or("Command ran successfully but there was no response.".to_string());

    Ok(())
}

async fn is_interaction_from_admin(ctx: &AppContext<'_>) -> anyhow::Result<bool> {
    let Some(member) = ctx.author_member().await else {
        return Err(anyhow::anyhow!(
            "Cannot get member from the interaction. Is user {} not a member of the server?",
            fdisplay(ctx.author())
        ));
    };

    Ok(member
        .permissions(ctx.cache())
        .is_ok_and(|p| p.administrator()))
}
