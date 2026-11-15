// `/tps` from the console: always allowed.
fn tps_console_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    if *ran || !command.trim_start_matches('/').split_whitespace().next().is_some_and(|r| r.eq_ignore_ascii_case("tps")) {
        return;
    }
    *ran = true;
    for line in tps_core::reply(game.native_tps(), &game.native_tick_times()) {
        game.native_feedback(horizon_client::text::Text::legacy(&line.replace('&', "\u{a7}")));
    }
}
