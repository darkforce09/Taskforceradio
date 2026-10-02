# Mute rules

The serious-mode mute rules and the record of the clients this plugin muted.

## Contents

```text
crates/version_1_0/tfar_radio_routing_1_0/src/mute/
├── killed_mute_matrix.rs  `killed_mute_matrix` and `MuteChanges`: who hears whom after a kill
├── mute_decision.rs       `mute_decision`: range, radio and liveness decide one client's mute
├── mute_tracker.rs        `MuteTracker`: the clients muted, so requests go out only on a change
└── tests/                 unit tests of each rule
```

## How it works

The plugin app asks `mute_decision` after a client's position, volume or tangent and
`killed_mute_matrix` after a kill, feeds the answers to its `MuteTracker`, and sends TeamSpeak only
the changes the tracker reports. On game end or disconnect `unmute_all` returns every client the
plugin muted; a client that leaves is forgotten with `client_left`.

## Boundaries

- Depends on: the crate's `reception` and `routing_context`; `tfar_client_state_1_0`.
- Used by: the crate root; the plugin app.
- Rules: the local player is never muted (`the_local_player_is_never_muted`); outside serious mode
  nothing is muted (`outside_serious_mode_every_other_client_is_unmuted`); the living hear only
  the living in serious mode (`in_serious_mode_the_living_hear_only_the_living`).
