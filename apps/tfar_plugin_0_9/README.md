# TFAR 0.9 TeamSpeak plugin

The `tfar_plugin_0_9` crate: the TeamSpeak 3 plugin for the 0.9.12 mod, built as a native Linux
shared library. TeamSpeak 3 loads it from its plugins folder.

## Contents

```text
apps/tfar_plugin_0_9/
├── Cargo.toml  the `tfar_plugin_0_9` package (cdylib + rlib): tier 6, workspace lints and dependencies
├── src/        the exports, the shared core, the game messages, the events, the voice callbacks
└── tests/      the `.so` in the mock host: game session, dormancy, game end, HRTF reset, shutdown
```

## How it works

Every domain decision lives in the 0.9 library crates; the app is wiring, threads and TeamSpeak
side effects. `ts3plugin_init` builds one `PluginCore` behind an `Arc`: TeamSpeak (through the
`TeamSpeak` trait), one `ServerConnection` per established server connection (the
`ServerRadioData`, the `MuteTracker`, the `VoiceProcessor` with its scratch and the channel's
voice facts behind one `parking_lot::Mutex`, beside a `PlaybackMixer`), the `GameSession`, the
radio sounds with the decoded remote sounds, the HRIR sphere, the local push-to-talk and the
running web requests. Then it starts the game link server for the 0.9 flavour on both its ports,
47810 and the `-tfdebug` port 47811 (the newest valid `HELLO` on either holds the session), with
the `GameLinkBridge`, the 100 ms service thread, the push-to-talk release thread and the update
check.

```text
Arma ──game link──▶ GameLinkBridge ──▶ parse (tfar_game_commands_0_9) ──▶ lock, apply, decide
                                                                        └▶ unlock, TeamSpeak calls
TeamSpeak events ──▶ PluginCore (nickname list, plugin commands, VOLUME broadcast, info panel)
TeamSpeak audio  ──▶ try_lock ──▶ VoiceProcessor / PlaybackMixer ──▶ samples (pass through if busy)
service thread   ──▶ game timeout, 4 s sweep and metadata, finished sounds freed
```

- **Dormant outside a game.** Until a message that counts as game activity arrives (anything but
  `VERSION`), the plugin writes no metadata, nickname, mute, channel move or plugin command, and
  every audio callback passes through. When the game times out (4 s without game activity) the
  plugin ends it (every mute lifted, the off sound, the channel and nickname given back, the
  metadata set to "not in game" once) and is dormant again until the next game activity, so the
  0.9 and 1.0 plugins can both stay installed
  ([D25](/documentation/decisions/game_link.md#2026-10-03--a-plugin-returns-to-dormant-after-its-game-end-handling-d25)).
  The game link stays bound and answers while dormant.
- **Own voice and remote sounds.** The captured-voice callback pushes the own voice into the
  mixer's `my_radio_voice` stream through a `StreamProducer` that never blocks or allocates; a
  remote push-to-talk sound is shaped by `render_remote_tangent_sound` of the voice pipeline. When
  a client starts talking (`onTalkStatusChangeEvent`), its HRTF direction sources are reset, so the
  tail of its last utterance never replays.
- **Locks.** Outermost first: the push-to-talk lock, the server map, one connection's state; the
  game session mutex alone. No TeamSpeak call runs under the map or a state lock: each command reads
  TeamSpeak, locks to decide, unlocks, then calls TeamSpeak. The voice callbacks only `try_read` and
  `try_lock`, and read the TeamSpeak facts they need (our id, each client's channel name and plugin
  metadata) from the state instead of calling TeamSpeak.
- **Sound state** is made on the command threads with `ensure_dsp(dsp_requirements(..))` after
  every `POS`, `FREQ`, `SPEAKERS`, push-to-talk, `VOLUME` and game start; the audio thread only
  borrows it.
- **Shutdown** stays within TeamSpeak's 2 s: it stops the link server and cancels the web
  requests, joins the service and release threads, then ends a running game (way back from the
  serious channel, nickname given back), writes the disabled status when awake, lifts every mute,
  clears the state, joins the web requests until 1.5 s after the start, and drops the log
  forwarder last. A web request whose network call is still in flight then cannot be interrupted:
  it is cancelled (its answer is dropped), logged, and ends within its own timeout (10 s for the
  update check, 5 s for a tracking request) after `ts3plugin_shutdown` returned.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_plugin_0_9   # the unit tests, then the .so in the mock host
cargo build -p tfar_plugin_0_9  # libtfar_plugin_0_9.so under target/debug/
cargo run -p tfar_ts3_mock -- target/debug/libtfar_plugin_0_9.so  # load it by hand in the mock host
```

## Configuration

Environment variables of the TeamSpeak process, read once in `ts3plugin_init`:

- `TFAR_GAME_LINK_PORT`: for tests only. A port from 1 to 65535 makes the plugin listen on that one
  port alone, with no debug port, so test suites that load the plugin never fight over the fixed
  ports; any other value is ignored with a warning in TeamSpeak's log, and the plugin listens on
  both 47810 and 47811 (`src/listen_port.rs`).
- `TFAR_SKIP_UPDATE_CHECK`: for tests only. `1` skips the update check, so no test reaches the
  network (`src/running_plugin.rs`).

From TeamSpeak: the push-to-talk delay of the capture profile in `settings.db` under TeamSpeak's
config folder, read once at init; from the plugins folder: `radio-sounds/` and
`tfar_hrir/IRC_1002_C.bin`. Without the sphere the HRTF is off, voices are placed by the 0.9 simple
ILD, and the chat tab says so.

## Exports

- Symbols: every `ts3plugin_*` export `ts3_plugin!` writes: name, version, API version, author,
  description, function pointers, init, shutdown, offers-configure, plugin ID, command keyword and
  command, info title, data and free, autoload, and the event and audio callbacks, with the original
  0.9.12 metadata (name `Task Force Arma 3 Radio`, version `0.9.12`, author `[TF]Nkey`, info title
  `Task Force Radio Status (0.9.12)`, no command keyword, API 26, autoload, no configure dialog).
- Artifact: task_force_radio_linux_amd64.so, which `cargo xtask dist` copies from
  libtfar_plugin_0_9.so into target/dist/tfar_0_9/ beside radio-sounds/ and tfar_hrir/; its plugin
  name starts with `task_force_radio`, the prefix every 0.9.12 plugin accepts commands from.
- Installed by: the owner, by hand, into the Flatpak TeamSpeak plugins folder
  (~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/); TeamSpeak then lists the plugin in its
  plugin settings.

## Boundaries

- Depends on: `tfar_ts3_host` (exports, function table, log forwarder); `tfar_ts3_settings_db`
  (push-to-talk delay); `tfar_game_link_server` (the listener); `tfar_web_requests` (update check,
  Piwik); `tfar_game_commands_0_9`, `tfar_plugin_commands_0_9`, `tfar_client_state_0_9`,
  `tfar_radio_routing_0_9`, `tfar_voice_pipeline_0_9` (every 0.9 decision); `tfar_playback_mixer`,
  `tfar_hrtf`, `tfar_sample_buffer`, `tfar_dsp_filters`, `tfar_spatial_audio` (sounds and voice
  buffers); `tfar_geometry`, `tfar_arma_values`; `parking_lot`, `crossbeam-channel`, `tracing`,
  `thiserror`. In tests, `tfar_ts3_mock` and `tfar_game_link_client`.
- Used by: TeamSpeak 3, which loads the shared library; `tools/tfar_ts3_mock` in the plugin test
  suites.
- Rules: a dormant plugin writes nothing to TeamSpeak and passes audio through
  (`tests/plugin_scenario_in_mock_host.rs`, `without_game_activity_the_plugin_stays_dormant`), and
  is dormant again for 10 s of service ticks after a game end until the game resumes
  (`tests/plugin_game_end_and_shutdown_in_mock_host.rs`,
  `after_the_game_ends_the_plugin_stays_dormant_until_the_game_resumes`; with the time injected in
  `src/tests/service_thread.rs`); `ts3plugin_shutdown` returns within 2 s with every plugin thread
  joined (`shutdown_joins_every_plugin_thread_within_two_seconds`), only a web request still in
  flight outlives it (`a_web_request_in_flight_never_holds_the_shutdown_past_two_seconds`), and no
  TeamSpeak memory leaks (`leak_report` in the host tests). Tier 6 in `apps`: every workspace dependency has a lower tier and follows the edge matrix
  in `crates/README.md` (`cargo xtask verify crate-tiers`). `unsafe` is allowed here for the export
  glue only (`cargo xtask verify unsafe-allowlist`).

## Related documentation

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — the ports, the
  handshake, the dormant rule and the plugin side of the link.
- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — the 0.9 voice chain and the
  callbacks.
- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
- [Bug fixes](/documentation/program/bug_fixes.md) — the original bugs the port fixes.
