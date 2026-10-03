# Service

The plugin's background housekeeping: the 100 ms service thread.

## Contents

```text
apps/tfar_plugin_1_0/src/service/
├── mod.rs              `mod` lines and the re-export of `ServiceThread`
└── service_thread.rs   `ServiceThread`: the game link poll, the 4 s sweep, the debug snapshots
```

## How it works

Every tick the thread polls the game link (timeout, debounced transitions) and frees finished
radio sounds. Unless the plugin is dormant, every 4 s it moves into the serious channel while in
game (focus-gated unless `moveWhileTabbedOut`, never for a spectator with
`noAutomoveSpectator`), expires stale clients, recomputes every mute, reads the client metadata,
builds the DSP objects and writes the own metadata; every 500 ms it publishes a debug window
snapshot while the window is open.

## Boundaries

- Depends on: `crossbeam-channel` (the stop signal); the crate's `runtime` and `diagnostics`.
- Used by: `plugin_lifecycle`, which starts it, and `runtime`, which stops it at shutdown.
- Rules: a stop request ends the loop within one tick; the thread holds only a weak runtime handle.
