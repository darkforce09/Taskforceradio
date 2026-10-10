# Game link

The plugin side of the game link: the handler the link server calls, the sync answers, the async
messages, the local tangent and the settings.

## Contents

```text
crates/version_1_0/tfar_plugin_1_0/src/game_link/
├── async_commands.rs   the async messages applied in order: POS, FREQ, KILLED, DFRAME, SPEAKERS, towers, TRACK
├── link_handler.rs     `LinkHandler`: the `GameLinkHandler` of the running plugin
├── local_tangent.rs    the own `TANGENT` and `RELEASE_ALL_TANGENTS`: session apply, sounds, push-to-talk
├── mod.rs              `mod` lines and the re-export of `LinkHandler`
├── settings_apply.rs   `SETCFG`: the settings, needcfg, serious mode and the version warning
└── sync_replies.rs     the answers to `TS_INFO`, `POS`, `IS_SPEAKING` and `IS_SPEAKING_BULK`
```

## How it works

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

## Boundaries

- Depends on: `tfar_game_link_server`, `tfar_game_commands_1_0`, `tfar_plugin_commands_1_0`,
  `tfar_plugin_settings_1_0`, `tfar_client_state_1_0`, `tfar_antennas_1_0`, `tfar_web_requests`;
  the crate's `runtime`, `audio`, `push_to_talk` and `diagnostics`.
- Used by: `plugin_lifecycle`, which hands `LinkHandler` to the link server.
- Rules: a malformed message gets its command's error reply and changes nothing; a message for a
  server without state is dropped.
