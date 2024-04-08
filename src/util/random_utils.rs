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
