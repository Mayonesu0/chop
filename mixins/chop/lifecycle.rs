// A world opened or closed: its Chop! data loads or is saved.
fn chop_world_lifecycle(
    game: &mut crate::game::Game<'_>,
    phase: crate::game::native_lifecycle::WorldPhase,
) {
    crate::game::chop_server::lifecycle(game, phase);
}
