# TFAR 1.0 TeamSpeak plugin

The `tfar_plugin_1_0` crate: the TeamSpeak 3 plugin for the 1.0 mod, built as a native Linux shared
library. TeamSpeak 3 loads it from its plugins folder.

## How it works

The TeamSpeak plugin for the 1.0 mod, wiring the 1.0 library crates to TeamSpeak:

- **Game link.** `ts3plugin_init` starts a `GameLinkServer` for the 1.0 flavour on both
  127.0.0.1:47820 and the `-tfdebug` port 47821; the newest valid `HELLO` on either port holds the
  session. Sync calls (`TS_INFO`, `POS`, `IS_SPEAKING`, `IS_SPEAKING_BULK`) are answered on the
  session thread; a sync `POS` is queued with `GameLinkServer::queue_async` and applied on the
  async worker, as the original queues it. Async messages (`POS`, `FREQ`, `KILLED`, `DFRAME`,
  `SPEAKERS`, the tangents, `SETCFG`, `MISSIONEND`, `RadioTwrAdd`/`RadioTwrDel`, `TRACK`,
  `collectDebugInfo`) run in arrival order on the async worker. `SETCFG` goes through
  `tfar_plugin_settings_1_0` and pushes needcfg.
- **State.** One `ServerState` per connection behind a `parking_lot` mutex: the 1.0 session, the
  mute tracker, the radio towers, the voice processor and its scratch. Apply functions run under
  the lock; TeamSpeak is called after it is released.
- **Game lifecycle.** The game counts as connected while its last message is younger than
  `pluginTimeout`; the connect plays `radio-sounds/on` and moves into the serious channel, the
  disconnect lifts every mute, plays `off`, restores channel, nickname and voice activation and
  sets needcfg.
- **Audio.** The post-process callback runs `tfar_voice_pipeline_1_0`, the captured callback feeds
  speaker radios through the mixer's own-voice `StreamProducer`, the mixed-playback callback adds
  the radio sounds. The voice callbacks wait for the server lock, as the original waits on its
  critical section, since a block passed through unprocessed is an audible jump; no holder calls
  TeamSpeak or takes another lock under it, and the waits are logged at debug level once a second
  when any occurred. A remote tangent sound is shaped by `render_remote_tangent_sound`. An HRTF
  direction source restarts itself after 200 ms without input (`tfar_hrtf`), so an utterance
  never replays the last one's tail and no other thread cuts into a running one.
- **Plugin folder.** `init` loads `radio-sounds/` and `tfar_hrir/` from `Ts3::plugin_folder`:
  `getPluginPath` under the plugin ID TeamSpeak registered last (its "Reload" keeps the `.so`
  loaded but registers a new ID, and the plugin commands go out under it too), else the folder of
  the loaded `.so` (`tests/plugin_reload_in_mock_host.rs`).
- **Push-to-talk.** A local tangent goes through `SessionState::apply_local_tangent`, then forces
  the microphone on and broadcasts to the channel; the release waits TeamSpeak's push-to-talk
  delay plus `tangentReleaseDelay` on its own thread.
- **Service.** Every 100 ms the game link poll; every 4 s the serious-channel move (focus-gated),
  the expiry sweep, every mute, the client metadata and the own metadata.
- **Web requests.** The update check and each `TRACK` run as cancellable `BackgroundRequest`s;
  shutdown cancels them first and joins them last within 1 s. A request already waiting for the
  network is not interrupted: one still running at the deadline is logged and left, cancelled, and
  ends within its own timeout (10 s for the update check, 5 s for tracking) without acting.
- **Diagnostics.** `/tfar diag|pos|full|debug|rstflt`, the debug window and the dump.
- **Dormant.** Until the first game message, and again after each game end has been handled
  (mutes lifted, `off` played, channel and nickname back, the not-in-game metadata written once),
  the plugin sends nothing and passes audio through; the next game message wakes it.

`plugin_exports.rs` expands `ts3_plugin!(PLUGIN)`. `TfarPlugin::init` installs the log forwarder,
loads the radio sounds and the HRIR sphere, reads TeamSpeak's push-to-talk delay, builds the
`PluginRuntime`, starts the `GameLinkServer` on both ports with a `LinkHandler`, the service
thread and the cancellable update check, and publishes the runtime in an `ArcSwapOption`. Every
callback loads it: the event callbacks call runtime methods, the audio callbacks the functions of
`audio::voice_callbacks`, which wait only for a server lock whose holders never call TeamSpeak. All TeamSpeak traffic goes through `TeamSpeakActions`; side effects only
through `PluginRuntime::act`, which is `None` while the plugin is dormant. `TfarPlugin::shutdown`
takes the runtime out, runs `PluginRuntime::shut_down` (web requests cancelled, game end, metadata
cleared, mutes lifted, link server, service, release thread and debug window stopped and joined,
web requests joined within 1 s) and drops the log forwarder last, all within 2 s.

## Modules

### `src/audio/`

The post-process callback waits for the server state (counting the wait in the runtime's
`AudioDiagnostics`), copies the block to stereo, runs `VoiceProcessor::process_sender` and writes
the result back; no game, a dormant plugin or an unknown sender leaves the block untouched. The captured callback runs
`VoiceProcessor::process_own_voice_for_speakers` and pushes the result through the mixer's
`StreamProducer` for the stream `my_radio_voice`, which never blocks or allocates. The
mixed-playback callback adds the mixer's playbacks and streams into channels 0 and 1 of the
server in front. `RadioSounds` preloads `radio-sounds/` at init; a local tangent sound is queued
at its gain into its ear, a remote one is shaped by `render_remote_tangent_sound` of
`tfar_voice_pipeline_1_0` (gain, ear, vehicle and loudspeaker filters, under-water muffling,
simple ILD) on the calling thread before the mixer gets it.

### `src/diagnostics/`

Every report reads the runtime under short locks and calls no TeamSpeak function meanwhile. The
dump writes `diag.log`, `pos.log`, `settings.log` and `game_messages.log` into a folder named by
the UTC time under TeamSpeak's config path. `/tfar debug` starts the debug window's UI thread on
first use; without an X11 display it prints the reason in the chat tab instead.

### `src/events/`

`plugin_command_received` accepts only commands of the server in front from a plugin whose name
starts with `TFAR`, decodes them with `tfar_plugin_commands_1_0::decode` and applies them under the
server lock: a remote tangent through `tfar_radio_routing_1_0::apply_remote_tangent`, which yields
the remote sounds, the reception count and the heard list. The sounds are shaped and queued and the mute sent after the lock is released.
`own_variable_changed` sends `VOLUME` to the channel when the own talking flag changes in game.

### `src/game_link/`

`LinkHandler` holds a weak runtime handle. Every sync call and every async message except
`MISSIONEND` counts as a game message (`record_game_message`, which runs a due connect transition
first); `MISSIONEND` forgets the game's messages so the game disconnects. Sync calls are parsed
with `tfar_game_commands_1_0::parse` and answered from the fixed replies or the session; a `POS`
answers its speaking code and is queued with `GameLinkServer::queue_async` behind every async
message already queued, as the original queues it. Async messages, those queued `POS` calls
included, arrive from the link server's FIFO worker and go through `apply_async_message`; a
`TRACK` starts a cancellable Piwik request the runtime keeps until shutdown. The local tangent
goes through `SessionState::apply_local_tangent` and `apply_local_release_all_tangents`, then
push-to-talk and the broadcast.

### `src/push_to_talk/`

A press, unless a release is waiting, saves and switches off voice activation, sets
`CLIENT_INPUT_DEACTIVATED` to active and sends the broadcast; with a release waiting it marks that
release skipped. A release marks itself waiting and hands a job to `tfar-tangent-release`, which
waits `push_to_talk_delay + tangentReleaseDelay` (cut short only by shutdown) and then, unless
skipped, restores voice activation and the microphone and sends the broadcast. A game end cancels
every release still waiting (the gate's release generation moves on, so a job from before it does
nothing), and the plugin, dormant again after its game, never broadcasts a release.

### `src/runtime/`

`PluginRuntime` holds a `ServerRegistry` of `parking_lot::Mutex<ServerState>`, one per established
connection; a `ServerState` bundles the `SessionState`, the `MuteTracker`, the `AntennaManager`, the
`VoiceProcessor` with its stereo scratch, the metadata cache the voice gate reads, the channel
cache and the nickname and channel to restore. Every handler takes the lock, runs the apply
function of a library crate, collects what TeamSpeak must do, releases the lock and only then
calls `TeamSpeakActions`. `PluginRuntime::act` answers `None` while the plugin is dormant, so no
side effect reaches TeamSpeak before the first game message, nor after a game end (once its
handling ran) until the next one. The web requests live in `BackgroundRequests` until shutdown
cancels them and joins them within one budget. The game link state lives in its own leaf lock;
`poll_game_link` runs the transition it reports after releasing it.

### `src/service/`

Every tick the thread polls the game link (timeout, debounced transitions) and frees finished
radio sounds. Unless the plugin is dormant, every 4 s it moves into the serious channel while in
game (focus-gated unless `moveWhileTabbedOut`, never for a spectator with
`noAutomoveSpectator`), expires stale clients, recomputes every mute, reads the client metadata,
builds the DSP objects and writes the own metadata; every 500 ms it publishes a debug window
snapshot while the window is open. The periods are `ServicePeriods`; the test-only
`TFAR_SERVICE_TICK_MS` shortens the tick and scales the sweep (40 ticks) and the snapshot (5 ticks)
with it, so a test sees many seconds of ticks quickly.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_plugin_1_0   # the unit tests, then the .so in the mock host
cargo build -p tfar_plugin_1_0  # libtfar_plugin_1_0.so under target/debug/
cargo run -p tfar_ts3_mock -- target/debug/libtfar_plugin_1_0.so  # load it by hand in the mock host
```

## Configuration

Environment variables of the TeamSpeak process, read once in `ts3plugin_init`:

- `TFAR_GAME_LINK_PORT`: for tests only. A port from 1 to 65535 is the one port the plugin then
  listens on, instead of both 47820 and 47821, so test suites that load the plugin never fight
  over the fixed ports; any other value is ignored with a warning in TeamSpeak's log.
- `TFAR_SKIP_UPDATE_CHECK`: for tests only. `1` skips the update check, so a test without network
  sends no request.
- `TFAR_SERVICE_TICK_MS`: for tests only. 1 to 100 sets the service tick in milliseconds and
  scales the 4 s sweep (40 ticks) and the 500 ms snapshot (5 ticks) with it; the game link's
  timeout and debounce stay in real time. Any other value is ignored with a warning.

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
  `tfar_sample_buffer`; `tfar_arma_values`; `tfar_debug_window`; `parking_lot`, `arc-swap`, `crossbeam-channel`,
  `profiling`, `tracing`, `thiserror`, and `tracy-client` behind `tracy`. In tests,
  `tfar_ts3_mock` and `tfar_game_link_client`.
- Used by: TeamSpeak 3, which loads the shared library; `tools/tfar_ts3_mock` in the plugin test
  suites.
- Rules: a plugin is dormant, making no side effect and passing audio through untouched, until
  its first game message and again after each game end until the next one (see the game-link decisions,
  `after_its_game_end_the_plugin_is_dormant_until_game_activity_resumes`); no TeamSpeak call
  while a server lock is held; the audio callbacks never block; `ts3plugin_shutdown` returns
  within 2 s and joins every thread it started, except a web request still waiting for the
  network after its 1 s join budget (`shutdown_joins_every_thread_the_plugin_started_within_two_seconds`
  in `tests/game_scenarios.rs`). Tier 6 in `crates/version_1_0`: every workspace
  dependency has a lower tier and follows the category edge matrix
  (`cargo xtask verify crate-tiers`). `unsafe` is allowed here for the export glue only
  (`cargo xtask verify unsafe-allowlist`).

## Related documentation

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — the ports, the
  handshake and the plugin side of the link.
- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — the 1.0 voice chain the
  callbacks run.
- [TFAR 1.0 plugin behaviour](/documentation/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
- [Windows dependencies](/documentation/behaviour_specs/windows_dependencies.md) — the Win32
  objects the originals use and what replaces them.
