// A player starts digging a block: a log trembles while it is dug.
fn chop_left_click_block(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    face: [i32; 3],
    state: horizon_world::registry::BlockStateId,
) {
    crate::game::chop_server::left_click(game, pos, face, state);
}
