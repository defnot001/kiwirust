mod commands;
mod config;
mod events;
mod handler;
mod util;

use config::Config;
use handler::Handler;
use serenity::prelude::*;

#[tokio::main]
async fn main() {
    let token = Config::get().bot.bot_token.as_str();

    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Failed to create client");

    if let Err(e) = client.start().await {
        println!("Client error: {:?}", e);
    }
}
