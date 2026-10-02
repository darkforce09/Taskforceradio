# TFAR 1.0 TeamSpeak plugin

The `tfar_plugin_1_0` crate: the TeamSpeak 3 plugin for the 1.0 mod, built as a native Linux shared
library. TeamSpeak 3 loads it from its plugins folder.

## Contents

```text
apps/tfar_plugin_1_0/
├── Cargo.toml  the `tfar_plugin_1_0` package (cdylib + rlib): tier 6, workspace lints and dependencies
├── src/        the exports, the plugin lifecycle, the port choice and the stub game link handler
└── tests/      the built `.so` in the mock TeamSpeak host: metadata, a game link ping, shutdown
```

## How it works

The TeamSpeak plugin for the 1.0 mod: the `ts3plugin_*` exports, the game link listener with its
async queue, the TeamSpeak event handlers, antennas, `SETCFG`, the focus-aware channel move and the
debug window, wired to the 1.0 crates. Today it is the stub that proves the path end to end:

- `src/plugin_exports.rs` expands `ts3_plugin!` over the `static PLUGIN: TfarPlugin`, so every
  `ts3plugin_*` symbol exists with the original 1.0 metadata (name `Task Force Arrowhead Radio`,
  version `1.0.0.1`, author `[TF]Nkey and Dedmen`, description `Radio Addon for Arma 3`, info title
  `Task Force Radio Status`, command keyword `tfar`, API 26, autoload, no configure dialog).
- `ts3plugin_init` installs the TeamSpeak log forwarder and starts a `GameLinkServer` for the 1.0
  flavour on 127.0.0.1:47820 with `StubGameLinkHandler`, which answers `TS_INFO\tPING` with `PONG`
  and every other sync call with `UNKNOWN COMMAND`; async messages are ignored, so the plugin keeps
  reporting needcfg until the 1.0 command handling applies `SETCFG`.
- `ts3plugin_shutdown` stops the server, joining its threads, then drops the log forwarder.

Every other callback keeps `Ts3Plugin`'s no-op default, so the plugin passes audio through
untouched.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_plugin_1_0   # the unit tests, then the .so in the mock host
cargo build -p tfar_plugin_1_0  # libtfar_plugin_1_0.so under target/debug/
cargo run -p tfar_ts3_mock -- target/debug/libtfar_plugin_1_0.so  # load it by hand in the mock host
```

## Configuration

Two environment variables of the TeamSpeak process, read once in `ts3plugin_init`
(`src/listen_port.rs`):

- `TFAR_GAME_LINK_DEBUG`: `1` makes the plugin listen on the debug port 47821, the one a
  `-tfdebug` Arma instance connects to; unset or any other value keeps 47820.
- `TFAR_GAME_LINK_PORT`: for tests only. A port from 1 to 65535 overrides the port choice, so test
  suites that load the plugin never fight over the fixed ports; any other value is ignored with a
  warning in TeamSpeak's log.

## Exports

- Symbols: every `ts3plugin_*` export `ts3_plugin!` writes: name, version, API version, author,
  description, function pointers, init, shutdown, offers-configure, plugin ID, command keyword and
  command, info title, data and free, autoload, and the event and audio callbacks.
- Artifact: TFAR_linux_amd64.so, which `cargo xtask dist` copies from
  libtfar_plugin_1_0.so into target/dist/tfar_1_0/ beside radio-sounds/.
- Installed by: the owner, by hand, into the Flatpak TeamSpeak plugins folder
  (~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/); TeamSpeak then lists the plugin in its
  plugin settings.

## Boundaries

- Depends on: `tfar_ts3_host` (the safe TeamSpeak export layer and the log forwarder);
  `tfar_game_link_server` (the plugin-side link listener); `tracing` (the plugin's log lines);
  `thiserror` for the error type. Declared for the full plugin and not used by the stub yet:
  `tfar_ts3_settings_db`, `tfar_web_requests`, `tfar_voice_pipeline_1_0`,
  `tfar_radio_routing_1_0`, `tfar_client_state_1_0`, `tfar_game_commands_1_0`,
  `tfar_plugin_commands_1_0`, `tfar_playback_mixer`, `tfar_geometry`, `tfar_arma_values`,
  `tfar_plugin_settings_1_0`, `tfar_antennas_1_0`, `tfar_debug_window`. In tests,
  `tfar_ts3_mock` and `tfar_game_link_client`.
- Used by: TeamSpeak 3, which loads the shared library; `tools/tfar_ts3_mock` in the plugin test
  suites.
- Rules: a plugin whose game link never connected this session stays dormant and passes audio
  through untouched; no thread outlives `ts3plugin_shutdown` (`tests/plugin_in_mock_host.rs`).
  Tier 6 in `apps`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`). `unsafe` is allowed here for the export
  glue only (`cargo xtask verify unsafe-allowlist`).

## Related documentation

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — the ports, the
  handshake and the plugin side of the link.
- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
- [Windows dependencies](/documentation/program/behaviour_specs/windows_dependencies.md) — the Win32
  objects the originals use and what replaces them.
