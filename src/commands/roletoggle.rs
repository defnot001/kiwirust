use anyhow::Context;

use crate::{util::format::display, Context as AppContext};

#[derive(Debug, poise::ChoiceParameter)]
enum RoleChoice {
    KiwiInc,
    PingPong,
}

/// Toggle roles for yourself.
#[poise::command(slash_command, guild_only = true)]
pub async fn roletoggle(
    ctx: AppContext<'_>,
    #[description = "The role you want to toggle."] role_choice: RoleChoice,
) -> anyhow::Result<()> {
    ctx.defer_ephemeral().await?;

    let Some(member) = ctx.author_member().await else {
        tracing::error!(
            "Cannot get member from the interaction. Is user {} not a member of the server?",
            display(ctx.author())
        );

        return Err(anyhow::anyhow!("Cannot get member from the interaction!",));
    };

    let Some(guild) = ctx.partial_guild().await else {
        if let Some(guild_id) = ctx.guild_id() {
            tracing::error!("Cannot get guild with id {} from the interaction", guild_id);
        } else {
            tracing::error!("Cannot get guild from the interaction.");
        }

        return Err(anyhow::anyhow!("Cannot get guild from the interaction!"));
    };

    let role_id = match role_choice {
        RoleChoice::KiwiInc => ctx.data().config.roles.kiwi_inc,
        RoleChoice::PingPong => ctx.data().config.roles.pingpong,
    };

    if member.roles.contains(&role_id) {
        let res = member.remove_role(&ctx, role_id).await;

        if let Err(e) = res {
            tracing::error!(
                "Failed to remove role {:?} from {}: {e}",
                role_choice,
                display(ctx.author()),
            );

            return Err(e).context("Failed to remove role");
        }
    } else {
        let res = member.add_role(&ctx, role_id).await;

        if let Err(e) = res {
            tracing::error!(
                "Failed to add role {:?} to {}: {e}",
                role_choice,
                display(ctx.author()),
            );

            return Err(e).context("Failed to add role");
        }
    }

    match ctx
        .say(format!("Successfully toggled role {:?}!", role_choice))
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!(
                "Failed to send message after toggling role {:?} for {}: {e}",
                role_choice,
                display(ctx.author()),
            );

            Err(e).context("Failed to send message")
        }
    }
}
