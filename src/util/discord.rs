use serenity::{
    client::Context,
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
};

use crate::error::Error;

pub async fn edit_unknown_option_response(
    interaction: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), Error> {
    interaction
        .edit_original_interaction_response(&ctx.http, |message| message.content("Unknown option."))
        .await?;

    Ok(())
}

pub async fn edit_response_message(
    interaction: &ApplicationCommandInteraction,
    ctx: &Context,
    message: impl Into<String>,
) -> Result<(), Error> {
    let message = message.into();

    interaction
        .edit_original_interaction_response(&ctx.http, |msg| msg.content(message))
        .await?;

    Ok(())
}
