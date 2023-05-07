use serenity::{
    builder::CreateApplicationCommand,
    client::Context,
    model::prelude::{
        command::CommandOptionType,
        interaction::application_command::{
            ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
        },
    },
};

use crate::{error::Error, util::discord};

pub async fn execute(
    interaction: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
    ctx: &Context,
) -> Result<(), Error> {
    if let Some(animal_choice) = options.get(0) {
        let animal =
            animal_choice
                .resolved
                .as_ref()
                .ok_or(Error::Serenity(serenity::Error::Other(
                    "Expected animal option",
                )))?;

        if let CommandDataOptionValue::String(animal) = animal {
            let animal = animal.as_str();

            let url = match animal {
                "fox" => "https://randomfox.ca/floof/",
                "cat" => "https://api.thecatapi.com/v1/images/search",
                "dog" => "https://api.thedogapi.com/v1/images/search",
                _ => {
                    return discord::edit_unknown_option_response(interaction, ctx).await;
                }
            };

            let response = reqwest::get(url).await?.json::<serde_json::Value>().await?;

            let image_url = match animal {
                "fox" => response["image"].as_str(),
                "cat" => response[0]["url"].as_str(),
                "dog" => response[0]["url"].as_str(),
                _ => unreachable!(),
            }
            .ok_or_else(|| Error::Serenity(serenity::Error::Other("Failed to get image url.")))?;

            discord::edit_response_message(interaction, ctx, image_url).await?;
        } else {
            return discord::edit_unknown_option_response(interaction, ctx).await;
        }
    } else {
        return discord::edit_unknown_option_response(interaction, ctx).await;
    }

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("animal")
        .description("Get random pictures from animals.")
        .create_option(|option| {
            option
                .name("animal")
                .description("Select an animal.")
                .kind(CommandOptionType::String)
                .required(true)
                .add_string_choice("Fox", "fox")
                .add_string_choice("Cat", "cat")
                .add_string_choice("Dog", "dog")
        })
}
