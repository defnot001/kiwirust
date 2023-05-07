use serenity::{
    model::{gateway::Ready, prelude::GuildId},
    prelude::Context,
};

use crate::{commands, config::Config};

pub async fn handle_ready(ctx: Context, ready: Ready) {
    println!("Bot is ready. Logged in as {}!", ready.user.tag());

    let guild_id = Config::get().guild.guild_id;

    let commands =
        GuildId::set_application_commands(&guild_id, &ctx.http, commands::register_commands)
            .await
            .expect("Failed to register application commands");

    let partial_guild = guild_id
        .to_partial_guild(&ctx.http)
        .await
        .expect("Failed to get guild.");

    println!(
        "Successfully registered {} commands to {}.",
        commands.len(),
        partial_guild.name,
    );
}
