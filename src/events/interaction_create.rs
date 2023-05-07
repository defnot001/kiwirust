use serenity::client::Context;
use serenity::model::application::interaction::Interaction;

use crate::commands;

pub async fn handle_interaction_create(ctx: Context, interaction: Interaction) {
    if let Interaction::ApplicationCommand(interaction) = interaction {
        // println!("Received command interaction: {:#?}", command);

        if let Some(channel) = interaction
            .channel_id
            .to_channel(&ctx.http)
            .await
            .ok()
            .and_then(|channel| channel.guild())
        {
            println!(
                "{} triggered a command in #{}.",
                interaction.user.tag(),
                channel.name
            )
        } else {
            println!(
                "{} triggered a command. Could not get the channel name.",
                interaction.user.tag(),
            );
        }

        let result = commands::run_command(&interaction, &ctx).await;

        if let Err(e) = result {
            println!("Interaction Error: {:?}", e);
        }
    }
}
