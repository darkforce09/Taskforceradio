# TFAR 1.0 TeamSpeak plugin

The `tfar_plugin_1_0` crate: the TeamSpeak 3 plugin for the 1.0 mod, built as a native Linux shared
library. TeamSpeak 3 loads it from its plugins folder.

## Contents

```text
apps/tfar_plugin_1_0/
├── Cargo.toml  the `tfar_plugin_1_0` package (cdylib + rlib): tier 6, workspace lints and dependencies
├── src/        the exports, the lifecycle, the runtime, the game link handler, events, audio, services
└── tests/      the built `.so` in the mock TeamSpeak host, driven by a real game link client
```

## How it works

The TeamSpeak plugin for the 1.0 mod, wiring the 1.0 library crates to TeamSpeak:

- **Game link.** `ts3plugin_init` starts a `GameLinkServer` for the 1.0 flavour on 127.0.0.1:47820.
  Sync calls (`TS_INFO`, `POS`, `IS_SPEAKING`, `IS_SPEAKING_BULK`) are answered on the session
  thread; async messages (`POS`, `FREQ`, `KILLED`, `DFRAME`, `SPEAKERS`, the tangents, `SETCFG`,
  `MISSIONEND`, `RadioTwrAdd`/`RadioTwrDel`, `TRACK`, `collectDebugInfo`) run in arrival order on
  the async worker. `SETCFG` goes through `tfar_plugin_settings_1_0` and pushes needcfg.
- **State.** One `ServerState` per connection behind a `parking_lot` mutex: the 1.0 session, the
  mute tracker, the radio towers, the voice processor and its scratch. Apply functions run under
  the lock; TeamSpeak is called after it is released.
- **Game lifecycle.** The game counts as connected while its last message is younger than
  `pluginTimeout`; the connect plays `radio-sounds/on` and moves into the serious channel, the
  disconnect lifts every mute, plays `off`, restores channel, nickname and voice activation and
  sets needcfg.
- **Audio.** The post-process callback runs `tfar_voice_pipeline_1_0`, the captured callback feeds
  speaker radios, the mixed-playback callback adds the radio sounds; all three only `try_lock`.
- **Push-to-talk.** A local tangent forces the microphone on and broadcasts to the channel; the
  release waits TeamSpeak's push-to-talk delay plus `tangentReleaseDelay` on its own thread.
- **Service.** Every 100 ms the game link poll; every 4 s the serious-channel move (focus-gated),
  the expiry sweep, every mute, the client metadata and the own metadata.
- **Diagnostics.** `/tfar diag|pos|full|debug|rstflt`, the debug window and the dump.
- **Dormant.** Until the first game message the plugin sends nothing and passes audio through.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_plugin_1_0   # the unit tests, then the .so in the mock host
cargo build -p tfar_plugin_1_0  # libtfar_plugin_1_0.so under target/debug/
cargo run -p tfar_ts3_mock -- target/debug/libtfar_plugin_1_0.so  # load it by hand in the mock host
```

## Configuration

Environment variables of the TeamSpeak process, read once in `ts3plugin_init`:

- `TFAR_GAME_LINK_DEBUG`: `1` makes the plugin listen on the debug port 47821, the one a
  `-tfdebug` Arma instance connects to; unset or any other value keeps 47820.
- `TFAR_GAME_LINK_PORT`: for tests only. A port from 1 to 65535 overrides the port choice, so test
  suites that load the plugin never fight over the fixed ports; any other value is ignored with a
  warning in TeamSpeak's log.
- `TFAR_SKIP_UPDATE_CHECK`: for tests only. `1` skips the update check, so a test without network
  starts no thread that outlives the plugin.

Files under TeamSpeak's plugins folder: `radio-sounds/` (the WAVs) and
`tfar_hrir/IRC_1002_C.bin` (the HRIR sphere; without it HRTF is off and the chat tab says so).
TeamSpeak's `settings.db` gives the push-to-talk delay.

Cargo features, off by default: `tracy` (profiling zones to a Tracy viewer through `profiling` and
`tracy-client`) and `api_profiler` (profiling zones as `tracing` spans).

## Exports

- Symbols: every `ts3plugin_*` export `ts3_plugin!` writes: name, version, API version, author,
  description, function pointers, init, shutdown, offers-configure, plugin ID, command keyword and
  command, info title, data and free, autoload, and the event and audio callbacks.
- Artifact: TFAR_linux_amd64.so, which `cargo xtask dist` copies from
  libtfar_plugin_1_0.so into target/dist/tfar_1_0/ beside radio-sounds/ and tfar_hrir/.
- Installed by: the owner, by hand, into the Flatpak TeamSpeak plugins folder
  (~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/); TeamSpeak then lists the plugin in its
  plugin settings.

## Boundaries

- Depends on: `tfar_ts3_host` (the export layer, the function table, the log forwarder);
  `tfar_ts3_settings_db` (the push-to-talk delay); `tfar_game_link_server`; `tfar_web_requests`
  (update check, Piwik); `tfar_voice_pipeline_1_0`, `tfar_radio_routing_1_0`,
  `tfar_client_state_1_0`, `tfar_game_commands_1_0`, `tfar_plugin_commands_1_0`,
  `tfar_plugin_settings_1_0`, `tfar_antennas_1_0`; `tfar_playback_mixer`, `tfar_hrtf`,
  `tfar_sample_buffer`, `tfar_dsp_filters`, `tfar_spatial_audio`; `tfar_geometry`,
  `tfar_arma_values`; `tfar_debug_window`; `parking_lot`, `arc-swap`, `crossbeam-channel`,
  `profiling`, `tracing`, `thiserror`, and `tracy-client` behind `tracy`. In tests,
  `tfar_ts3_mock` and `tfar_game_link_client`.
- Used by: TeamSpeak 3, which loads the shared library; `tools/tfar_ts3_mock` in the plugin test
  suites.
- Rules: a plugin whose game link never carried a game message this session stays dormant and
  passes audio through untouched; no TeamSpeak call while a server lock is held; the audio
  callbacks never block; no thread outlives `ts3plugin_shutdown` except an update check still
  waiting for the network (`tests/game_scenarios.rs`). Tier 6 in `apps`: every workspace
  dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`). `unsafe` is allowed here for the export glue only
  (`cargo xtask verify unsafe-allowlist`).

## Related documentation

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — the ports, the
  handshake and the plugin side of the link.
- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — the 1.0 voice chain the
  callbacks run.
- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
- [Windows dependencies](/documentation/program/behaviour_specs/windows_dependencies.md) — the Win32
  objects the originals use and what replaces them.
