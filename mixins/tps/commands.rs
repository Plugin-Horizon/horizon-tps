fn tps_register_commands(commands: &mut Vec<horizon_client::mod_api::NativeCommand>) {
    commands.push(horizon_client::mod_api::NativeCommand::op("tps", 2).permission("bukkit.command.tps"));
}
