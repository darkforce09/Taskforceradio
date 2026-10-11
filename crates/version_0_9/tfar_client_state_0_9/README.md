# Client state for 0.9

The `tfar_client_state_0_9` crate: the per-client state of the 0.9.12 plugin: positions, radios,
tangents and effect state, with the application of every game message and plugin command. Used by
`tfar_plugin_0_9`, `tfar_radio_routing_0_9`, `tfar_voice_pipeline_0_9`.

## How it works

The crate is pure state and decisions: it calls no TeamSpeak function, starts no thread and reads
no clock (every time comes in as `now: Instant`). The plugin app owns one `ServerRadioData` per
TeamSpeak server connection behind one `parking_lot::Mutex`, applies each parsed game message
(`tfar_game_commands_0_9`) and each decoded plugin command to it under the lock, and turns what the
`apply_*` methods return into TeamSpeak calls after releasing the lock.

`ServerRadioData` holds every field of the original's `SERVER_RADIO_DATA`
(`reference/tfar_0_9_12/ts/src/plugin.cpp:402`): the local player's nickname, original nickname,
push-to-talk, position, [SW](/documentation/glossary.md#sw) and [LR](/documentation/glossary.md#lr)
frequency maps, [DD](/documentation/glossary.md#dd) frequency, the
[speaker radios](/documentation/glossary.md#speaker-radio) by frequency, the levels, the
[serious mode](/documentation/glossary.md#serious-mode) channel and the current
[data frame](/documentation/glossary.md#data-frame). Its `clients` map holds one `ClientData` per
nickname, in nickname order as the original's `std::map`, with every `CLIENT_DATA` field
(plugin.cpp:192) and the sender's `ClientDsp`.

### Lock and ownership contract

- One lock per server connection guards the whole `ServerRadioData`, sound state included; no
  reference into it outlives the guard.
- Sound state is made only by `ClientData::ensure_dsp`, which the plugin calls while handling a
  command, with the `DspRequirements` that `tfar_radio_routing_0_9::dsp_requirements` computes for
  the sender. The audio thread waits for the lock, held only for short decisions, and only
  borrows what exists through the `_mut` accessors; it never makes or drops a filter, effect or
  HRTF source.
- `reset_radio_effects` drops sound state and `reset_voices` clears filter memory, both on the
  command threads; `remove_expired` hands the expired entries back, so the service thread frees
  their sound state after releasing the server lock.
- `GameSession` is one per plugin, beside the server states, as the original's global `inGame`.

### Fixed behaviour

- A game starts on the first message that counts as game activity, whatever the serious channel;
  serious mode only adds the channel move (`GameSession::record_message`,
  `ServerRadioData::start_game`).
- The original nickname is saved at the first successful rename of a game session and handed back
  once at game end (`confirm_rename`, `end_game`).
- A lookup by client id returns only an entry with a current data frame, and of several the one
  positioned last, so a rename's stale entry never stands in for the fresh one.
- An entry starts at the invalid data frame (9999) until its first `POS`, where the original starts
  it at frame 0 and so counts a player who never sent a position as current in the first frames of
  a game; the invalid frame is never current, also outside a game, where the current frame sits at
  or just above it (`an_entry_without_a_position_is_not_current_in_or_outside_a_game`,
  `a_killed_players_invalid_frame_is_not_current_outside_a_game`).
- Every field starts at a defined value, where the original leaves the plugin-enabled flag, its
  check time, the view angle, the local position, alive, can-speak and the levels uninitialised.
- A vehicle low-pass is redesigned when the isolation changes instead of one filter being kept per
  isolation until the voices reset.

### Shared ids

A loudspeaker radio's id and every vehicle descriptor (of a `ClientData` and of a `SpeakerRadio`)
are held behind an `Arc`, so a reception of `tfar_radio_routing_0_9` names them with a new handle
instead of a copy of the text, and the audio path routes without allocating. A `POS` replaces a
player's vehicle only when it changed.

```text
GameCommand / PluginCommand ──apply_*──▶ ServerRadioData ──clients──▶ ClientData ──dsp──▶ ClientDsp
        (game link, events)                │ data_frames: has_fresh_data, remove_expired
                                           │ game_session: start_game, end_game
GameSession ◀── record_message / check_timeout (game link thread, service tick)
DspRequirements (from the routing) ──ClientData::ensure_dsp──▶ ClientDsp (made off the audio thread)
```

`server_radio_data.rs` holds the state; `command_application/` adds the methods that apply each
command and return what the plugin does next (a reply, a rename, a sound, a mute check);
`data_frames.rs` and `nickname_list.rs` add the lookups and the client list. `client_data.rs` holds
one player, whose `ClientDsp` (`dsp_state/`) owns the compressor, direct-voice and per-radio sound
objects. `game_session.rs` turns the message traffic into the in-game flag.

## Modules

### `src/command_application/`

Each method takes the typed command (`tfar_game_commands_0_9` for the game, the decoded fields of
`tfar_plugin_commands_0_9` for the plugins) and `now` where the original stamps a time, changes the
state as `processGameCommand`, `processUnitPosition` and `processPluginCommand` in
`reference/tfar_0_9_12/ts/src/plugin.cpp` do, and returns an outcome naming the TeamSpeak side
effect: `PositionOutcome` (centre the listener, check the mute, or refresh the client list),
`RenameRequest`, the tangent-changed flag, `RemoteTangentOutcome` (route, mute and play the start or
end sound) and `RemoteVolumeOutcome`. None of them fails: an unknown nickname changes nothing and
says so.

### `src/dsp_state/`

The original makes every object on first use, on the TeamSpeak audio thread, under keys such as
`voice_clunk`, `local_cantspeak`, `local_vehicle` and the radio id
(`reference/tfar_0_9_12/ts/src/plugin.cpp:232-341`). Here `ClientDsp::ensure` makes them from a
`DspRequirements` while the plugin handles a command, so the audio thread only borrows: the direct
voice's objects sit in `DirectVoiceDsp`, the radio-keyed ones in a `RadioDsp` per radio id, and the
lookups by `&str` allocate nothing. The [HRTF](/documentation/glossary.md#hrtf) sources replace the
clunks: one for the direct voice and one per loudspeaker radio, made only when the plugin passes the
HRIR sphere. An existing object is never replaced, except a vehicle low-pass whose isolation
changed. `reset_radio_effects` drops the effects and the phone and loudspeaker band-passes;
`reset_voices` clears the memory of the can't-speak, under-water and vehicle low-passes and frees
nothing: in serious mode it runs on every `POS` of a silent player, under the server lock the
audio thread waits for, so it must not free or rebuild a two-megabyte direction source. The
direction sources restart themselves after 200 ms without input (`tfar_hrtf`); the compressor stays.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_client_state_0_9  # every command application, the session, expiry and sound state
```

## Configuration

None: the crate reads no setting. The HRIR sphere reaches it as an optional `&HrirLibrary` in
`ensure_dsp`; without one no direction source is made.

## Public surface

- `ServerRadioData` with `apply_position`, `apply_frequencies`, `confirm_rename`, `apply_version`,
  `apply_speakers`, `apply_killed`, `apply_local_tangent`, `apply_remote_tangent`,
  `apply_remote_volume`, `apply_release_all_tangents`, `update_nicknames`, `set_my_nickname`,
  `has_fresh_data`, `fresh_client_by_id`, `remove_expired`, `start_game`, `end_game`.
- `ClientData` with `ensure_dsp`, `reset_radio_effects`, `reset_voices`, `refresh_plugin_enabled`.
- `ClientDsp`, `DirectVoiceDsp`, `RadioDsp`, `VehicleFilter` and the request types
  `DspRequirements`, `DirectVoiceRequirement`, `RadioRequirement`, `SpeakerRequirement`,
  `RadioEffectKind`.
- `GameSession`, `GameStart`, `GameEnd`, `GameTimeout`, `SeriousChannel`.
- The outcome types `PositionOutcome`, `SpeakingState`, `RenameRequest`, `RemoteTangent`,
  `RemoteTangentOutcome`, `TangentSound`, `RemoteVolumeOutcome`.
- `ClientId`, `FrequencySettings`, `SpeakerRadio`, `VehicleMuffling`, the constants, `Error`,
  `Result`, `prelude`.

## Boundaries

- Depends on: `tfar_geometry` (positions and angles); `tfar_game_commands_0_9` (the typed game
  messages, `TangentKind`, `VehicleDescriptor`); `tfar_radio_effects_0_9` (the radio effects);
  `tfar_dsp_filters` (the Butterworth filters); `tfar_dynamics` (the compressor); `tfar_hrtf` (the
  direction sources); `thiserror` for the error type.
- Used by: `tfar_plugin_0_9`, `tfar_radio_routing_0_9`, `tfar_voice_pipeline_0_9`.
- Rules: no TeamSpeak call, thread or clock; sound state is made only off the audio thread
  (`src/dsp_state/tests/client_dsp.rs`); the fixed behaviours above are pinned by
  `in_game_without_a_serious_channel`, `the_original_nickname_after_two_renames`,
  `a_lookup_by_client_id_returns_the_fresh_entry_not_a_stale_one_left_by_a_rename` and
  `fields_the_original_leaves_uninitialised_start_at_defined_defaults`. Tier 3 in
  `crates/version_0_9`: every workspace dependency has a lower tier and follows the category edge matrix (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports; sections 1, 2 and 4.
- [Bug fixes](/documentation/architecture/bug_fixes.md) — the original bugs this crate fixes.
