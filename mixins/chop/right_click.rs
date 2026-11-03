// A player right-clicks a block: an empty hand on a trunk shakes the tree.
fn chop_right_click_block(game: &mut crate::game::Game<'_>, pos: [i32; 3], face: [i32; 3]) {
    crate::game::chop_server::right_click(game, pos, face);
}
