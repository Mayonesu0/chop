// Before a level's tick, with that level in place: its trees move, its cuts are looked at.
fn chop_level_tick_pre(game: &mut crate::game::Game<'_>, dimension: &str) {
    crate::game::chop_server::level_tick(game, dimension);
}
