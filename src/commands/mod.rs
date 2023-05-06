use serenity::builder::CreateApplicationCommands;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

macro_rules! commands {
    ($(mod $name:ident;)*) => {
        $(pub mod $name;)*

        pub fn run_command(name: &str, options: &[CommandDataOption]) -> String {
          match name {
            $(
              stringify!($name) => $name::run(options),
            )*
            _ => "not implemented".to_string(),
          }
        }

        pub fn register_commands(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
          commands $(.create_application_command(|command| $name::register(command)))*
        }
    };
}

commands! {
  mod attachmentinput;
  mod id;
  mod ping;
}
