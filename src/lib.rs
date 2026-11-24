// Chop! (c) 2026 Mayonesu0. All rights reserved, except the grants in LICENSE: https://github.com/Mayonesu0/chop/blob/main/LICENSE

//! Chop!: cut one log with an axe and the whole tree tips over, crashes down and breaks
//! apart. A native mod on Horizon's mod API alone: the hooks below hand each moment to the
//! game's side (`game`), which reads and changes the world through the host; the rules
//! (`rules`) decide what the tree is and how it falls, and know nothing of Horizon.

mod game;
mod rules;

use mod_api::{
    BlockBreakProgressUpdateEvent, BlockDamageAbortEvent, BreakBlockEvent, EntityPlaceEvent, Flow,
    Hooks, HostSide, LeftClickBlockEvent, LevelTickPreEvent, RegisterCommandsEvent, Registrar,
    RegistrationContext, RightClickBlockEvent, RunModCommandEvent, ServerHost, ServerStartedEvent,
    ServerStoppingEvent, ServerTickEvent, WorldLifecycleEvent,
};

#[derive(Default)]
struct Chop;

#[mod_api::horizon_mod]
impl Hooks for Chop {
    /// The particles Chop! sends, asked for once when the game loads.
    fn register(&self, registrar: &mut dyn Registrar, _: &RegistrationContext) {
        game::register(registrar);
    }

    /// A player breaks a block: a log is looked at on the next tick of its level.
    fn break_block(&self, host: &mut dyn ServerHost, event: &mut BreakBlockEvent) -> Flow {
        game::destroy(host, event.pos, event.state);
        Flow::Continue
    }

    /// A player starts digging a block: a log trembles while it is dug.
    fn left_click_block(&self, host: &mut dyn ServerHost, event: &mut LeftClickBlockEvent) -> Flow {
        game::left_click(host, event.pos, event.face, event.state);
        Flow::Continue
    }

    /// How far a dig has gone (the crack's stage, 0 to 9): the log trembles harder.
    fn block_break_progress_update(
        &self,
        host: &mut dyn ServerHost,
        event: &mut BlockBreakProgressUpdateEvent,
    ) {
        game::dig_progress(host, event.player, event.pos, event.progress);
    }

    /// A player stopped digging before the block broke: their log is let be.
    fn block_damage_abort(&self, host: &mut dyn ServerHost, event: &mut BlockDamageAbortEvent) {
        game::dig_stopped(host, event.player, event.pos);
    }

    /// A right click on a block: an empty hand on a trunk shakes the tree (server side only).
    fn right_click_block(&self, host: HostSide, event: &mut RightClickBlockEvent) -> Flow {
        if let HostSide::Server(host) = host {
            game::right_click(host, event.click);
        }
        Flow::Continue
    }

    /// A block is placed: the logs players place are never felled.
    fn entity_place(&self, host: &mut dyn ServerHost, event: &mut EntityPlaceEvent) -> Flow {
        game::place(host, event);
        Flow::Continue
    }

    /// Before a level's tick, with that level in place: its trees move, its cuts are looked at.
    fn level_tick_pre(&self, host: HostSide, event: &mut LevelTickPreEvent) {
        if let HostSide::Server(host) = host {
            game::level_tick(host, event.level);
        }
    }

    /// The start of each server tick: the clock, cuts that waited too long, saving.
    fn server_tick(&self, host: &mut dyn ServerHost, event: &mut ServerTickEvent) {
        game::server_tick(host, event.context, event.phase);
    }

    /// A world opened: its Chop! data loads.
    fn server_started(&self, host: &mut dyn ServerHost, _: &mut ServerStartedEvent) {
        game::opened(host);
    }

    /// The server stops: what still falls drops at once.
    fn server_stopping(&self, host: &mut dyn ServerHost, _: &mut ServerStoppingEvent) {
        game::stopping(host);
    }

    /// A world closed (or failed to open): its Chop! data is saved.
    fn world_lifecycle(&self, host: &mut dyn ServerHost, event: &mut WorldLifecycleEvent) {
        game::lifecycle(host, event.phase);
    }

    /// `/chop` in every player's command tree.
    fn register_commands(&self, _: &mut dyn ServerHost, event: &mut RegisterCommandsEvent) {
        game::commands(event.commands);
    }

    /// A player's `/chop ...`.
    fn run_mod_command(&self, host: &mut dyn ServerHost, event: &mut RunModCommandEvent) -> Flow {
        if game::perform(host, event.source, event.command) {
            Flow::Cancel
        } else {
            Flow::Continue
        }
    }
}
