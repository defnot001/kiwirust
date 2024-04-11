use std::borrow::BorrowMut;

use serenity::all::{ButtonStyle, CreateActionRow, CreateButton, CreateEmbed, PartialGuild};

pub fn sort_player_list(player_list: &mut [String]) {
    player_list.sort_by(|a, b| {
        let a_key = a
            .chars()
            .next()
            .map(|c| !c.is_alphanumeric())
            .unwrap_or(false);

        let b_key = b
            .chars()
            .next()
            .map(|c| !c.is_alphanumeric())
            .unwrap_or(false);

        if a_key == b_key {
            a.to_lowercase().cmp(&b.to_lowercase())
        } else {
            b_key.cmp(&a_key)
        }
    });
}

pub fn maybe_set_guild_thumbnail(embed: CreateEmbed, guild: &PartialGuild) -> CreateEmbed {
    if let Some(url) = guild.icon_url() {
        embed.thumbnail(url)
    } else {
        embed
    }
}

pub fn confirm_cancel_component() -> Vec<CreateActionRow> {
    let confirm = CreateButton::new("confirm")
        .label("Confirm")
        .style(ButtonStyle::Success);
    let cancel = CreateButton::new("cancel")
        .label("Cancel")
        .style(ButtonStyle::Danger);
    let action_row = CreateActionRow::Buttons(vec![confirm, cancel]);

    vec![action_row]
}
