// The start of each server tick: the clock, cuts that waited too long, saving.
fn chop_server_tick(
    game: &mut crate::game::Game<'_>,
    phase: crate::game::native_lifecycle::ServerTickPhase,
) {
    crate::game::chop_server::server_tick(game, phase);
}
