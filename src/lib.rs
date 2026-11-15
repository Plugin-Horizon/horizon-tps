//! `/tps` for a Horizon dedicated server as a native trait mod: the TPS of the last 1, 5 and 15
//! minutes (Paper's /tps line) plus the tick time of the last 100 ticks. The averaging lives in
//! the engine-free `tps-core` crate; this file only wires it to the tick hook and the command.
//!
//! Permission: node `bukkit.command.tps` (as on Paper), operator level 2 when no permissions mod
//! answers. The console may always run it.
use mod_api::{
    Command, Flow, Hooks, PlayerCommandEvent, PlayerRef, RegisterCommandsEvent, RunModCommandEvent,
    ServerHost, ServerTickEvent, ServerTickPhase, Text,
};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use tps_core::TpsTracker;

const NODE: &str = "bukkit.command.tps";
const LEVEL: u8 = 2;

#[derive(Default)]
struct Mod {
    tracker: Mutex<Option<TpsTracker>>,
}

/// Nanoseconds on a monotonic clock since the mod first asked.
fn now_ns() -> u64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_nanos() as u64
}

fn grants(host: &dyn ServerHost, player: PlayerRef, node: &str, level: u8) -> bool {
    player
        .permission_value(host, node)
        .unwrap_or_else(|| player.permission_level(host).unwrap_or(0) >= level)
}

fn coloured(host: &dyn ServerHost, line: &str) -> Text {
    host.text_from_legacy(line).unwrap_or_else(|_| Text::literal(line.to_owned()))
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn register_commands(&self, _: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        event.commands.push(Command::op("tps", LEVEL).with_node(NODE));
    }

    fn player_command(&self, host: &mut dyn ServerHost, event: &mut PlayerCommandEvent<'_>) -> Flow {
        // Without the node Paper answers with Bukkit's no-permission line, not "Unknown command".
        let root = event.command.trim_start_matches('/').split_whitespace().next().unwrap_or("").to_ascii_lowercase();
        if root != "tps" {
            return Flow::Continue;
        }
        if host.player(event.player).is_some_and(|p| grants(&*host, p, NODE, LEVEL)) {
            return Flow::Continue;
        }
        event.replies.push(Text::literal("\u{a7}cI'm sorry, but you do not have permission to perform this command. Please contact the server administrators if you believe that this is in error."));
        event.cancelled = true;
        Flow::Cancel
    }

    fn run_mod_command(&self, host: &mut dyn ServerHost, event: &mut RunModCommandEvent<'_>) -> Flow {
        if event.command.literals.first().map(String::as_str) != Some("tps") {
            return Flow::Continue;
        }
        let (rate, _) = host.tick_stats();
        let averages = self
            .tracker
            .lock()
            .map(|t| t.as_ref().map_or([f64::from(rate); 3], TpsTracker::averages))
            .unwrap_or([f64::from(rate); 3]);
        for line in tps_core::reply(averages, &host.tick_times()) {
            let text = coloured(&*host, &line);
            let _ = host.commands().feedback(text);
        }
        Flow::Cancel
    }

    fn server_tick(&self, host: &mut dyn ServerHost, event: &mut ServerTickEvent<'_>) {
        if !matches!(event.phase, ServerTickPhase::ServerTickStart { .. }) {
            return;
        }
        let rate = f64::from(host.tick_stats().0);
        if let Ok(mut tracker) = self.tracker.lock() {
            tracker.get_or_insert_with(|| TpsTracker::new(rate)).tick(now_ns(), rate);
        }
    }
}
