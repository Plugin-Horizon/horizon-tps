# horizon-tps

`/tps` for a [Horizon](https://github.com/horizon-rust/horizon) dedicated server, as a native trait mod
(arismoko mod API). Shows the TPS of the last 1, 5 and 15 minutes in Paper's format, plus the
average, lowest and highest tick time of the last 100 ticks:

```
TPS from last 1m, 5m, 15m: 20.0, 20.0, 19.8
Tick time (last 100 ticks) avg/min/max: 3.1/1.2/9.8 ms
```

- Permission node `bukkit.command.tps`; operator level 2 when no permissions mod answers. The console can always run it.
- Colours: TPS green from 18, yellow from 16, red below; tick time green up to 40 ms, yellow up to 50, red above.
- One TPS sample every 20 ticks; the averages start at the tick rate, so a fresh server reads 20.

## Layout

- `core/` — engine-free logic (rolling averages, reply text), std only: `cargo test` in `core/`.
- `src/lib.rs` — the hooks: `server_tick` feeds the tracker, `register_commands`/`run_mod_command` answer `/tps`.
- `horizon-mod.toml` — the engine revision it was built against.
- `mixins/tps/` — the same command for the older horizon-mixins line (glue injected into the engine); it reads the
  engine's own 1/5/15-minute averages (`native_tps`) instead of the trait mod's tracker.

## Build

Stage it with the engine like any native trait mod (`horizon-launcher native-build`), then build
`-p horizon-native-game --no-default-features`.

## License

MIT
