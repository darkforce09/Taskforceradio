# Runtime

The running 1.0 plugin: `PluginRuntime` with its state per server connection, and the work that
spans the game link and TeamSpeak (the game lifecycle, serious mode, the auto-mute and the client
roster).

## Contents

```text
apps/tfar_plugin_1_0/src/runtime/
├── background_requests.rs  the update check and tracking requests, cancelled and joined at shutdown
├── client_roster.rs        the own client and the clients of the own channel, their names and metadata
├── game_lifecycle.rs       the game link transitions, game start and end, the `on`/`off` sounds, own metadata
├── mod.rs                  `mod` lines and the re-exports of `PluginRuntime` and the server state
├── mute_updates.rs         the serious-mode auto-mute: decided under the lock, sent after it
├── plugin_runtime.rs       `PluginRuntime`: every piece of plugin state, the lock order, shutdown
├── serious_channel.rs      the channel cache, serious mode, the moves into the serious channel and back
└── server_state.rs         `ServerState` per connection and `ServerRegistry`
```

## How it works

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

## Boundaries

- Depends on: `tfar_client_state_1_0`, `tfar_radio_routing_1_0`, `tfar_voice_pipeline_1_0`,
  `tfar_antennas_1_0`, `tfar_plugin_settings_1_0`, `tfar_plugin_commands_1_0`,
  `tfar_game_link_server`, `tfar_debug_window`, `tfar_web_requests`, `tfar_ts3_host`; the
  crate's `teamspeak_actions`, `audio`, `push_to_talk`, `service` and `diagnostics`.
- Used by: `plugin_lifecycle`, `game_link`, `events`, `audio`, `service`, `diagnostics`.
- Rules: the lock order (registry, then one server lock; every other lock is a leaf) is stated in
  `plugin_runtime.rs`; no TeamSpeak call is made while a server lock is held; a dormant plugin
  makes no side effect (`a_dormant_plugin_makes_no_teamspeak_side_effect_and_passes_audio_through`,
  `after_its_game_end_the_plugin_is_dormant_until_game_activity_resumes`).
