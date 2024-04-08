use crate::{
    config::{Config, ServerChoice},
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

#[poise::command(slash_command, guild_only = true)]
async fn add(
    ctx: AppContext<'_>,
    #[description = "Add a player to the whitelist on all minecraft servers."] ign: String,
) -> anyhow::Result<()> {
    Ok(())
}

#[poise::command(slash_command, guild_only = true)]
async fn remove(
    ctx: AppContext<'_>,
    #[description = "Remove a player from the whitelist on all minecraft servers."] ign: String,
) -> anyhow::Result<()> {
    todo!()
}

#[poise::command(slash_command, guild_only = true)]
async fn list(
    ctx: AppContext<'_>,
    #[description = "The server to get the whitelist from."] server: ServerChoice,
) -> anyhow::Result<()> {
    ctx.defer().await?;

    let whitelist = match get_whitelist(&server, &ctx.data().config).await {
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

async fn get_whitelist(
    server_choice: &ServerChoice,
    config: &Config,
) -> anyhow::Result<Vec<String>> {
    let mut player_list = run_rcon_command(server_choice, config, vec!["whitelist list"])
        .await?
        .into_iter()
        .flatten()
        .next()
        .ok_or_else(|| anyhow::anyhow!("{server_choice} returned an unexpected or empty response"))?
        .split(": ")
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("{server_choice} returned an unexpected response"))?
        .split(", ")
        .map(str::to_string)
        .collect::<Vec<String>>();

    sort_player_list(&mut player_list);

    Ok(player_list)
}
