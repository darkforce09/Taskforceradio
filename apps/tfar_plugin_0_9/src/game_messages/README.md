# Game messages for the 0.9 plugin

The game's messages over the game link, from the link server's handler to the reply: the dispatch
of every 0.9.12 command, and the three with the most TeamSpeak side effects in files of their own.

## Contents

```text
apps/tfar_plugin_0_9/src/game_messages/
├── frequency_message.rs      `FREQ`: radios and levels, the rename to the game's nickname
├── game_link_bridge.rs       `GameLinkBridge`: the `GameLinkHandler` the link server calls
├── local_tangent_message.rs  `TANGENT*` from the game: local sounds, press and delayed release
├── message_dispatch.rs       parse, apply, start the game, reply; `TS_INFO`, `VERSION`, `KILLED` and the rest
├── mod.rs                    the module lines and the re-exports
├── position_message.rs       `POS`: the update, the serious-mode mute, the 3D neutralising, `SPEAKING`
└── tests/                    unit tests of the dispatch and the push-to-talk over the recording fake
```

## How it works

`GameLinkBridge::sync_request` decodes the text lossily and calls
`PluginCore::handle_game_message` on the link's session thread. A message that counts as game
activity wakes a dormant plugin first; `GameCommand::parse` types it (a refused message is `FAIL`);
the command is applied to the current server's `ServerState` under its lock with the library's
`apply_*` methods, and the TeamSpeak side effects follow with the lock released; then
`GameSession::record_message` starts the game on the first activity. `VERSION` that names a serious
channel after the game started makes the serious-mode move once, and a new addon version reaches
the metadata at once.

## Boundaries

- Depends on: `tfar_game_commands_0_9`, `tfar_plugin_commands_0_9`, `tfar_client_state_0_9`,
  `tfar_radio_routing_0_9`, `tfar_web_requests`; the crate's `plugin_core.rs`,
  `channel_snapshot.rs`, `radio_sounds.rs`, `push_to_talk/` and `game_session_transitions.rs`.
- Used by: `apps/tfar_plugin_0_9/src/running_plugin.rs`, which hands the bridge to the link server.
- Rules: every message gets one reply, the original's string (`tests/message_dispatch.rs`); a
  repeated push-to-talk sends nothing (`a_repeated_press_sends_nothing`); a mute TeamSpeak refused
  is asked again (`a_refused_mute_is_forgotten_and_asked_again`).
