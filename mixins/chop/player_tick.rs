// The end of a player's tick: a player who stopped digging lets their log be.
fn chop_player_tick(game: &mut crate::game::Game<'_>) {
    crate::game::chop_server::player_tick(game);
}
