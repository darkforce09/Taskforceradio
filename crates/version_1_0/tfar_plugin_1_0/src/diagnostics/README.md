# Diagnostics

The `/tfar` chat commands, the text reports, the dump of `collectDebugInfo`, the debug window
snapshot and the ring of recent game messages.

## Contents

```text
crates/version_1_0/tfar_plugin_1_0/src/diagnostics/
├── chat_commands.rs    `/tfar diag`, `pos`, `full…`, `debug` (with `allowDebugging`) and `rstflt`
├── debug_dump.rs       the dump under `<config path>/logs/<dd-mm_HH-MM-SS>/`
├── debug_snapshot.rs   the debug window's content, published every 500 ms while it is open
├── message_ring.rs     `MessageRing`: the latest game messages, bounded
├── mod.rs              `mod` lines
├── reports.rs          the text reports: state, positions, settings, game messages
└── tests/              unit tests of the ring and the dump folder names
```

## How it works

Every report reads the runtime under short locks and calls no TeamSpeak function meanwhile. The
dump writes `diag.log`, `pos.log`, `settings.log` and `game_messages.log` into a folder named by
the UTC time under TeamSpeak's config path. `/tfar debug` starts the debug window's UI thread on
first use; without an X11 display it prints the reason in the chat tab instead.

## Boundaries

- Depends on: `tfar_debug_window`, `tfar_plugin_settings_1_0`, `tfar_ts3_host`; the crate's
  `runtime`.
- Used by: `plugin_lifecycle` (chat commands), `game_link` (the dump), `service` (snapshots).
- Rules: a report never changes plugin state; the message ring never grows past its capacity.
