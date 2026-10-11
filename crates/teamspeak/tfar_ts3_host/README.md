# TeamSpeak host layer

The `tfar_ts3_host` crate: the safe layer over the TeamSpeak 3 plugin SDK (API 26): exports,
callbacks, function table, logging. Used by `tfar_plugin_0_9`, `tfar_plugin_1_0`.

## Contents

```text
crates/teamspeak/tfar_ts3_host/
├── Cargo.toml    the `tfar_ts3_host` package (lib): tier 0, workspace lints and dependencies
├── sdk_headers/  the official API 26 SDK headers the layout test parses
└── src/          the function-table handle, the export layer, the logging bridge and their tests
```

## How it works

A plugin app implements `Ts3Plugin` on a `static` and expands `ts3_plugin!` over it; the macro
generates every `ts3plugin_*` symbol both TFAR plugins need, as `extern "C"` shells that hand the
raw arguments to the crate's glue, which converts them (newtype IDs, lossy `&str`, bounded sample
slices) and calls the trait. `ts3plugin_setFunctionPointers` copies TeamSpeak's `TS3Functions`
table (the `#[repr(C)]` struct of `ts3plugin-sys`) into a process-wide `Ts3` handle; its methods
are the only way the apps call TeamSpeak, and they copy and free every string and list TeamSpeak
allocates. `install_logging` routes `tracing` through a bounded queue to a logger thread that calls
`logMessage`, so an audio thread never calls TeamSpeak. The layout of the `ts3plugin-sys` table is
checked against the headers in `sdk_headers/`: all 230 slots match the SDK order at plugin API 26,
and the crate uses its own `#[repr(C)]` mirror nowhere.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_ts3_host             # the layout, function-table, export and logging tests
cargo test -p tfar_ts3_host sdk_layout  # only the checks against the SDK headers
```

## Configuration

None: the crate reads no setting. TeamSpeak's own paths come from `Ts3::plugin_path` (the raw
`getPluginPath`), `Ts3::plugin_folder` (`getPluginPath`, else the folder of the loaded plugin
library; what the apps use) and `Ts3::config_path`.

## Public surface

- `ts3_plugin!` and `Ts3Plugin`: the exports and the callbacks a plugin app implements.
- `Ts3`: one safe method per TeamSpeak function either TFAR plugin calls, each returning
  `Result<_, Ts3Error>` where TeamSpeak can fail.
- `install_logging`, `LoggingGuard`, `print_to_current_tab`: logging into TeamSpeak and the chat
  tab.
- The SDK newtypes and values: `ServerConnectionHandlerId`, `ClientId`, `ChannelId`,
  `ConnectStatus`, `ClientProperty` and the rest listed in [the source README](/crates/teamspeak/tfar_ts3_host/src/README.md).

## Boundaries

- Depends on: no workspace crate; `ts3plugin-sys` (the raw table and SDK enums), `libc` (`malloc`
  and `free` for `infoData`, `dladdr` for the plugin folder), `arc-swap` (the plugin ID),
  `tracing`, `tracing-subscriber`, `crossbeam-channel`, `thiserror`.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`.
- Rules: no panic or unwind crosses into TeamSpeak; every C string handed over stays alive for the
  call; the `ts3plugin-sys` layout matches the SDK headers (`src/tests/sdk_layout.rs`). Tier 0 in
  `crates/teamspeak`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin and the TeamSpeak API it uses.
- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin and the TeamSpeak API it uses.
- [Library survey](/documentation/program/research/library_survey.md) — why `ts3plugin-sys` and
  not the high-level `ts3plugin` crate.
- [Coding standards](/documentation/standards/code_and_documentation.md) — the FFI and audio-thread
  rules this crate holds.
