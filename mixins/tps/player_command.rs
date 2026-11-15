// `/tps` from a player: node `bukkit.command.tps`, operator level 2 where no permissions mod answers.
fn tps_player_command(
    game: &mut crate::game::Game<'_>,
    player: &crate::game::native_server::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    if *ran || !command.trim_start_matches('/').split_whitespace().next().is_some_and(|r| r.eq_ignore_ascii_case("tps")) {
        return;
    }
    *ran = true;
    let lines = if game.native_has_permission(player.uuid, "bukkit.command.tps", 2) {
        tps_core::reply(game.native_tps(), &game.native_tick_times())
    } else {
        vec!["&cI'm sorry, but you do not have permission to perform this command. Please contact the server administrators if you believe that this is in error.".to_owned()]
    };
    for line in lines {
        game.native_feedback(horizon_client::text::Text::legacy(&line.replace('&', "\u{a7}")));
    }
}
