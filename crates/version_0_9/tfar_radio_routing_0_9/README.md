# Radio routing for 0.9

The `tfar_radio_routing_0_9` crate: whether and how a 0.9.12 listener hears a speaker: direct, over
radio or over a speaker radio, and which players the plugin mutes. Used by `tfar_plugin_0_9`,
`tfar_voice_pipeline_0_9`.

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

```text
ServerRadioData ──reception──▶ Vec<HeardVia> ──▶ voice pipeline (per frame)
       │                             ├──▶ mute_decisions ──MuteTracker──▶ MuteDecision ──▶ plugin
       │                             └──▶ dsp_planning ──▶ DspRequirements ──▶ ClientData::ensure_dsp
       └── distances (effective distance, diver reach, effect error)
```

`reception/` decides every way a sender reaches the listener, on `distances.rs`; `heard_via.rs`
holds the result type and the ignore flags. `mute_decisions/` and `dsp_planning.rs` both build on
the receptions. `serious_mode.rs` holds the channel-name checks the plugin applies to TeamSpeak's
channel names before it asks for a mute.

## Modules

### `src/mute_decisions/`

`game_client_mute` ports `setGameClientMuteStatus`
(`reference/tfar_0_9_12/ts/src/plugin.cpp:1188`): outside serious mode the player is wanted
unmuted; in it, muted unless both have current data, the listener is alive and the player is heard
over a radio or talks within voice range. `set_game_client_mute_status` also resets the voice state
of a player it mutes, and only of one with an entry: filter memory cleared, nothing freed, since
it runs on every `POS` of a silent player. `set_mute_for_dead_players` (plugin.cpp:1356)
wants every client without current data muted while the listener is alive in serious mode, and
unmuted otherwise. Each wanted state goes through the `MuteTracker`, which the plugin keeps per
server connection: it returns a mute request only for a client it has not muted, an unmute only for
one it muted, and `unmute_all` for exactly the tracked set; the plugin sends the requests after it
releases its lock, and calls `forget` when TeamSpeak refuses one or a client leaves.

### `src/reception/`

`is_over_local_radio` runs the checks of `isOverLocalRadio`
(`reference/tfar_0_9_12/ts/src/plugin.cpp:989`): short range on the listener's long-range list,
short range on short range, long range on short range, long range on long range, then diver; a
later match overwrites an earlier one. Short and long range need the family held (or its ignore
flag), the frequency on the list, both sides able to use their radio and the effective distance
below the sender's range; the diver radio compares the raw distance with its reach.
`is_over_radio` adds the local reception for another client, then, while the sender is in range on
a held short- or long-range radio it can use, one reception per loudspeaker radio on the sender's
frequency that the sender does not carry, at its position or at its owner's while the owner's data
is current. `is_over_radio_into` does the same into a list the caller reuses; every reception
shares its radio id and vehicle with the server state, so it copies no text.

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
- `dsp_requirements`, `Error`, `Result`, `prelude`.

## Boundaries

- Depends on: `tfar_client_state_0_9` (the state it reads and the sound requests it fills);
  `tfar_geometry` (positions); `thiserror` for the error type.
- Used by: `tfar_plugin_0_9`, `tfar_voice_pipeline_0_9`.
- Rules: the check order and the strict range comparisons of the original
  (`src/reception/tests/local_radio.rs`); an unmute only for a client this plugin muted
  (`outside_serious_mode_unmute_goes_only_to_clients_this_plugin_muted`); a missing entry is muted
  without touching voice state (`a_missing_entry_is_muted_without_touching_voice_state`); no
  function but the two `set_*` ones changes state. Tier 4 in `crates/version_0_9`: every workspace
  dependency has a lower tier and follows the category edge matrix
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports; sections 3 and 4.
- [Bug fixes](/documentation/architecture/bug_fixes.md) — the original bugs this crate fixes.
