# Mute decisions for 0.9

Which players the plugin mutes in TeamSpeak, and the requests that get them there from what the
plugin itself asked for.

## Contents

```text
crates/version_0_9/tfar_radio_routing_0_9/src/mute_decisions/
├── dead_players.rs      `set_mute_for_dead_players`: players without current data
├── game_client_mute.rs  `game_client_mute` and `set_game_client_mute_status`: one player in serious mode
├── mod.rs               the module lines and the re-exports
├── mute_tracker.rs      `MuteTracker` and `MuteDecision`: requests only on a change
└── tests/               unit tests per decision and for the tracker
```

## How it works

`game_client_mute` ports `setGameClientMuteStatus`
(`reference/tfar_0_9_12/ts/src/plugin.cpp:1188`): outside serious mode the player is wanted
unmuted; in it, muted unless both have current data, the listener is alive and the player is heard
over a radio or talks within voice range. `set_game_client_mute_status` also drops the voice state
of a player it mutes, and only of one with an entry. `set_mute_for_dead_players` (plugin.cpp:1356)
wants every client without current data muted while the listener is alive in serious mode, and
unmuted otherwise. Each wanted state goes through the `MuteTracker`, which the plugin keeps per
server connection: it returns a mute request only for a client it has not muted, an unmute only for
one it muted, and `unmute_all` for exactly the tracked set; the plugin sends the requests after it
releases its lock, and calls `forget` when TeamSpeak refuses one or a client leaves.

## Boundaries

- Depends on: `crates/version_0_9/tfar_radio_routing_0_9/src/reception/`,
  `crates/version_0_9/tfar_radio_routing_0_9/src/distances.rs`; `tfar_client_state_0_9`.
- Used by: `tfar_plugin_0_9`, after every `POS`, `VOLUME`, push-to-talk reception, `KILLED` and the
  4 s service tick.
- Rules: an unmute only for a client this plugin muted and a request only on a change
  (`outside_serious_mode_unmute_goes_only_to_clients_this_plugin_muted`); a missing entry muted
  without touching voice state (`a_missing_entry_is_muted_without_touching_voice_state`).
