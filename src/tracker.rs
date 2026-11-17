//! The engine-free part of the TPS mod: rolling 1/5/15-minute TPS averages fed from tick
//! timestamps, and the text `/tps` answers with. Std only, so it is tested without a game.

/// Ticks per sample: one TPS sample is taken every 20 ticks (about once a second).
pub const SAMPLE_TICKS: u64 = 20;
/// Samples kept per window: 1, 5 and 15 minutes at one sample a second.
pub const WINDOWS: [usize; 3] = [60, 300, 900];

/// One window: a ring buffer of samples with a running sum, so the average is O(1).
#[derive(Debug, Clone)]
struct Window {
    samples: Vec<f64>,
    next: usize,
    sum: f64,
}

impl Window {
    fn new(size: usize, fill: f64) -> Self {
        Self { samples: vec![fill; size], next: 0, sum: fill * size as f64 }
    }

    fn push(&mut self, value: f64) {
        self.sum += value - self.samples[self.next];
        self.samples[self.next] = value;
        self.next = (self.next + 1) % self.samples.len();
    }

    fn average(&self) -> f64 {
        self.sum / self.samples.len() as f64
    }
}

/// The rolling averages. Starts full of `rate` (20), so a fresh server reads 20, not 0.
#[derive(Debug, Clone)]
pub struct TpsTracker {
    windows: [Window; 3],
    ticks: u64,
    sample_start_ns: Option<u64>,
}

impl TpsTracker {
    pub fn new(rate: f64) -> Self {
        Self { windows: WINDOWS.map(|n| Window::new(n, rate)), ticks: 0, sample_start_ns: None }
    }

    /// Call at the start of every server tick with a monotonic clock in nanoseconds.
    /// `rate` is the server's target tick rate (20 unless `/tick rate` changed it).
    pub fn tick(&mut self, now_ns: u64, rate: f64) {
        let Some(start) = self.sample_start_ns else {
            self.sample_start_ns = Some(now_ns);
            return;
        };
        self.ticks += 1;
        if self.ticks < SAMPLE_TICKS {
            return;
        }
        let elapsed = now_ns.saturating_sub(start).max(1) as f64;
        let tps = (SAMPLE_TICKS as f64 * 1e9 / elapsed).min(rate.max(1.0));
        for w in &mut self.windows {
            w.push(tps);
        }
        self.ticks = 0;
        self.sample_start_ns = Some(now_ns);
    }

    /// The 1, 5 and 15 minute averages.
    pub fn averages(&self) -> [f64; 3] {
        [self.windows[0].average(), self.windows[1].average(), self.windows[2].average()]
    }
}

/// A TPS value with its legacy colour: green from 18, yellow from 16, red below.
pub fn coloured_tps(tps: f64) -> String {
    let colour = if tps >= 18.0 { "&a" } else if tps >= 16.0 { "&e" } else { "&c" };
    format!("{colour}{:.1}", tps)
}

/// A tick time in ms with its legacy colour: green up to 40 ms, yellow up to 50, red above.
pub fn coloured_mspt(ms: f64) -> String {
    let colour = if ms <= 40.0 { "&a" } else if ms <= 50.0 { "&e" } else { "&c" };
    format!("{colour}{:.1}", ms)
}

/// Average, lowest and highest of the recorded tick durations (ns, 0 = not recorded), in ms.
pub fn mspt_figures(tick_times_ns: &[u64]) -> Option<(f64, f64, f64)> {
    let ms: Vec<f64> = tick_times_ns.iter().filter(|&&t| t > 0).map(|&t| t as f64 / 1e6).collect();
    if ms.is_empty() {
        return None;
    }
    let avg = ms.iter().sum::<f64>() / ms.len() as f64;
    let min = ms.iter().copied().fold(f64::INFINITY, f64::min);
    let max = ms.iter().copied().fold(0.0, f64::max);
    Some((avg, min, max))
}

/// The lines `/tps` answers with, `&` colour codes included.
pub fn reply(averages: [f64; 3], tick_times_ns: &[u64]) -> Vec<String> {
    let tps: Vec<String> = averages.iter().map(|&t| coloured_tps(t)).collect();
    let mut lines = vec![format!("&6TPS from last 1m, 5m, 15m: {}", tps.join("&6, "))];
    if let Some((avg, min, max)) = mspt_figures(tick_times_ns) {
        lines.push(format!(
            "&6Tick time (last 100 ticks) avg/min/max: {}&6/{}&6/{}&6 ms",
            coloured_mspt(avg),
            coloured_mspt(min),
            coloured_mspt(max)
        ));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: u64 = 1_000_000;

    #[test]
    fn fresh_tracker_reads_twenty() {
        assert_eq!(TpsTracker::new(20.0).averages(), [20.0, 20.0, 20.0]);
    }

    #[test]
    fn half_speed_ticks_show_in_the_one_minute_window_first() {
        let mut t = TpsTracker::new(20.0);
        let mut now = 0;
        t.tick(now, 20.0);
        // 60 samples of 20 ticks at 100 ms each = 10 TPS.
        for _ in 0..60 * 20 {
            now += 100 * MS;
            t.tick(now, 20.0);
        }
        let [one, five, fifteen] = t.averages();
        assert!((one - 10.0).abs() < 1e-9, "{one}");
        assert!((five - 18.0).abs() < 1e-9, "{five}"); // 60 of 300 samples at 10
        assert!((fifteen - (20.0 * 840.0 + 10.0 * 60.0) / 900.0).abs() < 1e-9, "{fifteen}");
    }

    #[test]
    fn fast_ticks_are_capped_at_the_rate() {
        let mut t = TpsTracker::new(20.0);
        let mut now = 0;
        t.tick(now, 20.0);
        for _ in 0..20 * 20 {
            now += 10 * MS;
            t.tick(now, 20.0);
        }
        assert_eq!(t.averages()[0], 20.0);
    }

    #[test]
    fn colours_follow_the_thresholds() {
        assert_eq!(coloured_tps(19.96), "&a20.0");
        assert_eq!(coloured_tps(17.0), "&e17.0");
        assert_eq!(coloured_tps(9.5), "&c9.5");
        assert_eq!(coloured_mspt(55.0), "&c55.0");
    }

    #[test]
    fn reply_has_tps_and_tick_time_lines() {
        let mut times = vec![0u64; 100];
        times[0] = 2 * MS;
        times[1] = 4 * MS;
        let lines = reply([20.0, 19.0, 15.0], &times);
        assert_eq!(lines[0], "&6TPS from last 1m, 5m, 15m: &a20.0&6, &a19.0&6, &c15.0");
        assert_eq!(lines[1], "&6Tick time (last 100 ticks) avg/min/max: &a3.0&6/&a2.0&6/&a4.0&6 ms");
    }

    #[test]
    fn no_tick_times_gives_only_the_tps_line() {
        assert_eq!(reply([20.0; 3], &[0; 100]).len(), 1);
    }
}
