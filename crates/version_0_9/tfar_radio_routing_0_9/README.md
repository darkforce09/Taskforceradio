# Radio routing for 0.9

The `tfar_radio_routing_0_9` crate: whether and how a 0.9.12 listener hears a speaker: direct, over
radio or over a speaker radio, and which players the plugin mutes. Used by `tfar_plugin_0_9`,
`tfar_voice_pipeline_0_9`.

## Contents

```text
crates/version_0_9/tfar_radio_routing_0_9/
├── Cargo.toml  the `tfar_radio_routing_0_9` package (lib): tier 4, workspace lints and dependencies
└── src/        the distances, the reception rules, the mute decisions, serious mode and sound planning
```

## How it works

Every function reads a borrowed `tfar_client_state_0_9::ServerRadioData` under the plugin's lock
and calls no TeamSpeak function. `is_over_local_radio` and `is_over_radio` port `isOverLocalRadio`
and `isOverRadio` (`reference/tfar_0_9_12/ts/src/plugin.cpp:989`, `:1091`) check for check, a later
match overwriting an earlier one, and return one `HeardVia` per reception: the listener's own
[SW](/documentation/glossary.md#sw), [LR](/documentation/glossary.md#lr) or
[DD](/documentation/glossary.md#dd) radio first, then each
[speaker radio](/documentation/glossary.md#speaker-radio) on the sender's frequency.
`is_over_radio_into` fills a list the caller reuses instead, which it clears first: each
`HeardVia` shares its radio id and vehicle with the server state (`Arc`, and `local_radio_id` for
the listener's own radios), so a list with room for every reception routes a frame without
allocating; `is_over_radio` wraps it. `IgnoreTangents::for_kind` routes an incoming push-to-talk
whose
[tangent](/documentation/glossary.md#tangent) is already released, so its end sound still finds
its listeners.

The mute decisions port `setGameClientMuteStatus` and `setMuteForDeadPlayers` (plugin.cpp:1188,
:1356) as wanted states, and a `MuteTracker` per connection turns them into requests: a mute only
when the client is not muted yet, an unmute only for a client this plugin muted, and none for
client id 0. `unmute_all` unmutes exactly the tracked set. A player without a current entry is
muted without touching any voice state.

`dsp_requirements` walks the branches of the stereo function (plugin.cpp:2672-2797) and names the
sound objects the sender's voice will pass through; the plugin hands them to
`ClientData::ensure_dsp` while it handles a command.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_radio_routing_0_9  # every reception rule, the mute decisions, the sound planning
```

## Configuration

None: the crate reads no setting.

## Public surface

- `is_over_local_radio`, `is_over_radio`, `is_over_radio_into`, `receptions`, `HeardVia`,
  `HeardOn`, `IgnoreTangents`, `LOCAL_RADIO_ID`, `local_radio_id`.
- `raw_distance`, `effective_distance`, `diver_radio_distance`, `effect_error_from_distance` and
  the diver reach constants.
- `MuteTracker`, `MuteDecision`, `MuteQuery`, `GameClientMute`, `game_client_mute`,
  `set_game_client_mute_status`, `set_mute_for_dead_players`.
- `is_serious_mode`, `is_other_radio_plugin_channel`, `OTHER_RADIO_PLUGIN_CHANNELS`.
- `dsp_requirements`, `Error`, `Result`, `CRATE_ROLE`, `prelude`.

## Boundaries

- Depends on: `tfar_client_state_0_9` (the state it reads and the sound requests it fills);
  `tfar_geometry` (positions); `thiserror` for the error type.
- Used by: `tfar_plugin_0_9`, `tfar_voice_pipeline_0_9`.
- Rules: the check order and the strict range comparisons of the original
  (`src/reception/tests/local_radio.rs`); an unmute only for a client this plugin muted
  (`outside_serious_mode_unmute_goes_only_to_clients_this_plugin_muted`); a missing entry is muted
  without touching voice state (`a_missing_entry_is_muted_without_touching_voice_state`); no
  function but the two `set_*` ones changes state. Tier 4 in `crates/version_0_9`: every workspace
  dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports; sections 3 and 4.
- [Bug fixes](/documentation/program/bug_fixes.md) — the original bugs this crate fixes.
