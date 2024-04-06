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
        let message = format!(
            "Cannot get member from the interaction. User {} might not be a member of the guild",
            display(ctx.author())
        );
        tracing::error!("{}", message);
        ctx.say(message).await?;

        return Ok(());
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

            ctx.say("Failed to remove role").await?;
            return Ok(());
        }
    } else {
        let res = member.add_role(&ctx, role_id).await;

        if let Err(e) = res {
            tracing::error!(
                "Failed to add role {:?} to {}: {e}",
                role_choice,
                display(ctx.author()),
            );

            ctx.say("Failed to add role").await?;
            return Ok(());
        }
    }

    ctx.say(format!("Successfully toggled role {:?}!", role_choice))
        .await?;

    Ok(())
}
