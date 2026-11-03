// A player breaks a block: a log is looked at on the next tick of its level.
fn chop_destroy_block(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    state: horizon_world::registry::BlockStateId,
) {
    crate::game::chop_server::destroy(game, pos, state);
}
