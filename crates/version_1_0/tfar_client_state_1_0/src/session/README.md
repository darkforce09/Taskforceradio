# Session

`SessionState`, the state the plugin app keeps behind one lock per server connection, and the
apply function of each game and plugin message.

## Contents

```text
crates/version_1_0/tfar_client_state_1_0/src/session/
├── game_messages.rs    `apply_position`, `apply_frequencies`, `apply_killed`, `apply_speakers`
├── remote_commands.rs  `apply_remote_volume`, `apply_release_all_tangents`
├── remote_tangent.rs   `apply_remote_tangent`, `TangentEffects` and the remote sound placements
├── session_state.rs    `SessionState`: queries, the expiry sweep and the plugin-enabled check
└── tests/              unit tests of each module
```

## How it works

Each apply function takes a parsed message (from `tfar_game_commands_1_0` or
`tfar_plugin_commands_1_0`) and `now`, writes it into the session, and returns what the app does
next in TeamSpeak. `apply_remote_tangent` takes the reception routing as a closure so this crate
stays below the routing crate: for a release the closure runs before the sender's state is reset,
with the sender's abilities as the ignore flags; for a press after. Only a real press or release
plays a remote sound, and the local player's received transmissions count each sender once.

## Boundaries

- Depends on: the crate's `client_data`, `client_directory`, `game_data`, `reception`,
  `radio_volume`; `tfar_game_commands_1_0`, `tfar_plugin_commands_1_0`, `tfar_arma_values`,
  `tfar_geometry`.
- Used by: the crate root; the plugin app and `tfar_radio_routing_1_0::apply_remote_tangent`.
- Rules: a repeated press changes no sound and no count
  (`a_repeated_pressed_plays_no_sound_and_changes_no_count`); a release is routed before the reset
  (`a_release_is_routed_before_the_reset_with_the_senders_abilities`); expiry happens only in the
  sweep (`the_sweep_expires_stale_clients_and_names_them`).
