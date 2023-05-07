use serenity::model::gateway::Ready;
use serenity::{
    async_trait,
    model::prelude::interaction::Interaction,
    prelude::{Context, EventHandler},
};

use crate::events::{interaction_create, ready};
pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        interaction_create::handle_interaction_create(ctx, interaction).await
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        ready::handle_ready(ctx, ready).await
    }
}
