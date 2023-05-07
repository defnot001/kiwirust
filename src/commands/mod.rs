use serenity::builder::CreateApplicationCommands;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;

use crate::error::Error;

macro_rules! commands {
    ($(mod $name:ident;)*) => {
        $(pub mod $name;)*

        pub async fn run_command(interaction: &ApplicationCommandInteraction, ctx: &Context) -> Result<(), Error> {
          let name = interaction.data.name.as_str();

          match name {
            $(
              stringify!($name) => $name::execute(interaction, &interaction.data.options, ctx).await,
            )*
            _ => Err(Error::Serenity(serenity::Error::Other("Unknown Interaction."))),
          }
        }

        pub fn register_commands(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
          commands $(.create_application_command(|command| $name::register(command)))*
        }
    };
}

commands! {
  mod animal;
}
