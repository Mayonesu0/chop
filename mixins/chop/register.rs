// `/chop` in every player's command tree.
fn chop_register_commands(commands: &mut Vec<crate::game::native_server::NativeCommand>) {
    crate::game::chop_server::register(commands);
}
