// A player placed a block: the logs players place are never felled.
fn chop_place_block(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    state: horizon_world::registry::BlockStateId,
) {
    crate::game::chop_server::place(game, pos, state);
}
