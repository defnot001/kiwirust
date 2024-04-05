#![allow(dead_code, unused)]

mod commands;
mod config;
mod events;
mod util;

use commands::{animal, help, roletoggle, run};
use config::Config;
use events::event_handler;

use poise::serenity_prelude as serenity;
use sqlx::postgres::PgPoolOptions;

#[derive(Debug, Clone)]
pub struct Data {
    db_pool: sqlx::PgPool,
    config: Config,
}

pub type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::fmt().compact().finish())?;
    tracing::info!("Logger initialized.");

    let config = Config::load()?;
    tracing::info!("Config loaded.");

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database.url)
        .await?;
    tracing::info!("Database connected.");

    let client_intents = serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_MODERATION
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_MESSAGE_REACTIONS
        | serenity::GatewayIntents::GUILD_EMOJIS_AND_STICKERS;

    let register_guild_id = serenity::GuildId::from(config.bot.guild_id.clone());
    let bot_token = config.bot.token.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                animal::animal(),
                help::help(),
                run::run(),
                roletoggle::roletoggle(),
            ],
            event_handler: |ctx, event, framework, _data| {
                Box::pin(event_handler(ctx, event, framework))
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    register_guild_id,
                )
                .await?;
                Ok(Data { db_pool, config })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(bot_token, client_intents)
        .framework(framework)
        .await;

    client?.start().await?;

    Ok(())
}
