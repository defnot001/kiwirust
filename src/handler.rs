use serenity::model::{gateway::Ready, id::GuildId};
use serenity::{
    async_trait,
    model::prelude::interaction::{Interaction, InteractionResponseType},
    prelude::{Context, EventHandler},
};

use crate::commands;
use crate::config::Config;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            // println!("Received command interaction: {:#?}", command);

            if let Some(channel) = command
                .channel_id
                .to_channel(&ctx.http)
                .await
                .ok()
                .and_then(|channel| channel.guild())
            {
                println!(
                    "{} triggered a command in #{}.",
                    command.user.tag(),
                    channel.name
                )
            } else {
                println!(
                    "{} triggered a command. Could not get the channel name.",
                    command.user.tag(),
                );
            }

            let content = commands::run_command(&command.data.name, &command.data.options);

            if let Err(e) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", e);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
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
}
