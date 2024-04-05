use serenity::all::{CreateEmbed, CreateEmbedFooter, User};

use crate::Context;

pub async fn default_embed(user: &User) -> CreateEmbed {
    let footer = CreateEmbedFooter::new(format!(
        "Requested by {}",
        user.to_owned().global_name.unwrap_or(user.to_owned().name)
    ))
    .icon_url(user.static_avatar_url().unwrap_or_default());

    CreateEmbed::new()
        .color(3_517_048)
        .footer(footer)
        .timestamp(chrono::Utc::now())
}
