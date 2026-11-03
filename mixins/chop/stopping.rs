// The server stops: what still falls drops at once.
fn chop_server_stopping(game: &mut crate::game::Game<'_>) {
    crate::game::chop_server::stopping(game);
}
