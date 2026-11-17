# horizon-tps

`/tps` for a [Horizon](https://github.com/horizon-rust/horizon) dedicated server, as a native mod in the
flat `.hmod` layout. Shows the TPS of the last 1, 5 and 15 minutes in Paper's format, plus the
average, lowest and highest tick time of the last 100 ticks:

```
TPS from last 1m, 5m, 15m: 20.0, 20.0, 19.8
Tick time (last 100 ticks) avg/min/max: 3.1/1.2/9.8 ms
```

- Permission node `bukkit.command.tps`; operator level 2 when no permissions mod answers. The console can always run it.
- Colours: TPS green from 18, yellow from 16, red below; tick time green up to 40 ms, yellow up to 50, red above.
- One TPS sample every 20 ticks; the averages start at the tick rate, so a fresh server reads 20.

## Layout

- `src/tracker.rs` — engine-free logic (rolling averages, reply text), std only, with its unit tests.
- `src/lib.rs` — the hooks: `permission_gather_nodes` declares the node, `server_tick` feeds the tracker,
  `register_commands`/`run_mod_command` answer `/tps`.
- `horizon-mod.toml` — mod identity and the exact Horizon and horizon-mixins commits it is pinned to.

## Build

```sh
cargo horizon-mod build                          # target/horizon-mod/tps.hmod
cargo horizon-mod check target/horizon-mod/tps.hmod
```

Needs a signed SDK for the pinned build and Horizon's typed permission nodes (horizon-rust/horizon#17831).

## License

MIT
