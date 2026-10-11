# TFAR 0.9 TeamSpeak plugin

The `tfar_plugin_0_9` crate: the TeamSpeak 3 plugin for the 0.9.12 mod, built as a native Linux
shared library. TeamSpeak 3 loads it from its plugins folder.

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
TeamSpeak audio  ──▶ lock (waits) ──▶ VoiceProcessor / PlaybackMixer ──▶ samples
service thread   ──▶ game timeout, 4 s sweep and metadata, finished sounds freed
```

- **Dormant outside a game.** Until a message that counts as game activity arrives (anything but
  `VERSION`), the plugin writes no metadata, nickname, mute, channel move or plugin command, and
  every audio callback passes through. When the game times out (4 s without game activity) the
  plugin ends it (every mute lifted, the off sound, the channel and nickname given back, the
  metadata set to "not in game" once) and is dormant again until the next game activity, so the
  0.9 and 1.0 plugins can both stay installed
  ([decision](/documentation/decisions/game_link.md#2026-10-03--a-plugin-returns-to-dormant-after-its-game-end-handling-d25)).
  The game link stays bound and answers while dormant.
- **Plugin folder.** `init` loads `radio-sounds/` and `tfar_hrir/` from `Ts3::plugin_folder`:
  `getPluginPath` under the plugin ID TeamSpeak registered last (its "Reload" keeps the `.so`
  loaded but registers a new ID, and the plugin commands go out under it too), else the folder of
  the loaded `.so` (`tests/plugin_reload_in_mock_host.rs`).
- **Own voice and remote sounds.** The captured-voice callback pushes the own voice into the
  mixer's `my_radio_voice` stream through a `StreamProducer` that never blocks or allocates; a
  remote push-to-talk sound is shaped by `render_remote_tangent_sound` of the voice pipeline. When
  a client starts talking (`onTalkStatusChangeEvent`), its HRTF direction sources are reset, so the
  tail of its last utterance never replays.
- **Locks.** Outermost first: the push-to-talk lock, the server map, one connection's state; the
  game session mutex alone. No TeamSpeak call runs under the map or a state lock: each command reads
  TeamSpeak, locks to decide, unlocks, then calls TeamSpeak. The voice callbacks `try_read` the map
  and wait for a state, as the original waits on its critical section: a block passed through
  unprocessed between processed ones is an audible jump. The waits and skipped blocks are counted
  and logged at debug level once a second when any occurred. The callbacks read the TeamSpeak
  facts they need (our id, each client's channel name and plugin
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

`plugin_exports.rs` expands `ts3_plugin!(PLUGIN)`; `TfarPlugin` (in `plugin_lifecycle.rs`) keeps a
`RunningPlugin` behind a mutex and the `PluginCore` behind a `parking_lot::RwLock`. Main-thread
callbacks clone the core's `Arc` and call it; the audio callbacks `try_read` the lock and pass
through while `init` or `shutdown` holds it.

`PluginCore` methods are spread by concern over the modules: `game_messages/` for the game link,
`incoming_plugin_commands.rs` and `team_speak_events.rs` for TeamSpeak's events,
`audio_callbacks.rs` for the voice blocks, `service_thread.rs` for the tick,
`game_session_transitions.rs` and `status_metadata.rs` for the game's start, end and status. Every
one follows the same pattern: read TeamSpeak through the `TeamSpeak` traits, lock the connection's
`ServerState` to apply and decide with the library crates, unlock, then call TeamSpeak
(`send_mute_decision` forgets a mute TeamSpeak refused). The voice callbacks read
`TeamSpeakFacts`, which `channel_snapshot.rs` fills on every channel or client event and sweep.
A remote push-to-talk sound is shaped by the voice pipeline's `render_remote_tangent_sound`; the
own voice reaches the mixer through the `StreamProducer` each `ServerState` holds.

Threads: TeamSpeak's main thread (lifecycle, events, plugin commands), its audio threads (voice
callbacks), the game link server's accept and session threads (game messages), `tfar-service`,
`tfar-push-to-talk-release`, the web request threads (`tfar-update-check`, `tfar-piwik-track`)
and the log forwarder. The unit tests drive the core over `tests/fake_team_speak.rs`, a recording
fake of both traits, and the service tick with the time injected (`ServiceSchedule`); each module
attaches its sibling test file with `#[path]`.

## Modules

### `src/game_messages/`

`GameLinkBridge::sync_request` decodes the text lossily and calls
`PluginCore::handle_game_message` on the link's session thread. A message that counts as game
activity first wakes a dormant plugin and stamps the game session, both under the session lock
(`PluginCore::record_game_activity`), so it never races the service thread's return to dormant
after a game end; `GameCommand::parse` types it (a refused message is `FAIL`); the command is
applied to the current server's `ServerState` under its lock with the library's `apply_*`
methods, and the TeamSpeak side effects follow with the lock released; then the game starts when
this activity began one. `VERSION` that names a serious channel after the game started makes the
serious-mode move once, and a new addon version reaches the metadata at once. `TRACK` starts a
cancellable Piwik request kept in the core's `BackgroundWebRequests` for shutdown to join.

### `src/push_to_talk/`

A press with no release waiting saves the `vad` pre-processor state, switches voice activation off
when it was on, switches the microphone on (`CLIENT_INPUT_DEACTIVATED` = `INPUT_ACTIVE`, flushed)
and broadcasts the `TANGENT…PRESSED` command; a press while a release waits cancels that release and
sends nothing, since the others never saw it. A release is scheduled with TeamSpeak's push-to-talk
delay; the `tfar-push-to-talk-release` thread waits on a condvar until it is due, then restores voice
activation, switches the microphone off unless voice activation keeps it open, and broadcasts the
release. The lock is held across those TeamSpeak calls so a press and a release never interleave;
no audio callback and no TeamSpeak event takes it. A game end cancels a waiting release: voice
activation and the microphone are restored at once and nothing is broadcast, so a plugin dormant
again after its game sends no release. On stop, a waiting release is sent at once.

### `src/team_speak/`

`PluginCore` holds an `Arc<dyn TeamSpeak>`: `Ts3TeamSpeak` in the plugin, the recording fake in the
unit tests. A failed read becomes the neutral answer the original falls back to (`None`, an empty
list, `false`) with a debug line; a failed write returns TeamSpeak's error so the caller can react
(a refused mute is forgotten by the tracker).

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
  TeamSpeak memory leaks (`leak_report` in the host tests). Tier 6 in `crates/version_0_9`: every workspace dependency has a lower tier and follows the category edge matrix (`cargo xtask verify crate-tiers`). `unsafe` is allowed here for the export
  glue only (`cargo xtask verify unsafe-allowlist`).

## Related documentation

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — the ports, the
  handshake, the dormant rule and the plugin side of the link.
- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — the 0.9 voice chain and the
  callbacks.
- [TFAR 0.9.12 plugin behaviour](/documentation/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
- [Bug fixes](/documentation/architecture/bug_fixes.md) — the original bugs the port fixes.
