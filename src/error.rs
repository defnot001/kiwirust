use poise::FrameworkError;

use crate::Context as AppContext;
use crate::Data;

pub async fn error_handler<'a>(error: FrameworkError<'a, Data, anyhow::Error>) {
    match error {
        FrameworkError::Command { error, ctx, .. } => {
            ctx.reply("There was an error trying to execute that command.")
                .await
                .map_err(|e| tracing::error!("Failed to send error message: {:?}", e));

            tracing::error!("Command error: {:?}", error);
        }
        FrameworkError::CommandPanic { payload, ctx, .. } => {
            ctx.reply("Oops, something went terribly wrong. Please try again later.")
                .await
                .map_err(|e| tracing::error!("Failed to send error message: {:?}", e));

            tracing::error!("Command panic: {:?}", payload);
        }
        FrameworkError::GuildOnly { ctx, .. } => {
            ctx.reply("This command can only be used in a server.")
                .await
                .map_err(|e| tracing::error!("Failed to send error message: {:?}", e));

            tracing::error!(
                "Guild-only command {} was used outside of a guild.",
                ctx.command().name.clone()
            );
        }
        FrameworkError::SubcommandRequired { ctx } => {
            ctx.reply("This command requires a subcommand.")
                .await
                .map_err(|e| tracing::error!("Failed to send error message: {:?}", e));

            tracing::error!(
                "Command {} requires a subcommand but none was provided.",
                ctx.command().name.clone()
            );
        }
        FrameworkError::EventHandler { error, event, .. } => {
            tracing::error!(
                "Event handler error for {}: {:#?}",
                event.snake_case_name(),
                error
            );
        }
        FrameworkError::Setup {
            error,
            data_about_bot,
            ..
        } => {
            let username = data_about_bot.user.name.clone();
            tracing::error!("Failed to setup framework for {username}: {:#?}", error);
        }
        other => {
            tracing::error!("Unhandled framework error: {:?}", other);
        }
    }
}

pub async fn respond_error(
    message: impl AsRef<str>,
    error: impl std::fmt::Debug,
    context: &AppContext<'_>,
) -> anyhow::Result<()> {
    let message = message.as_ref();

    tracing::error!("{}: {:#?}", message, error);
    context.say(format!("{}.", message)).await?;

    Ok(())
}
