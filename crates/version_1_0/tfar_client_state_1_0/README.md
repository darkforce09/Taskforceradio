# Client state for 1.0

The `tfar_client_state_1_0` crate: the per-client state of the 1.0 plugin: positions, radios,
tangents, vehicles and effect state, the local player's game data, the game link's connection
state, and the apply function of each game and plugin message. Used by `tfar_plugin_1_0`,
`tfar_radio_routing_1_0`, `tfar_voice_pipeline_1_0`.

## Contents

```text
crates/version_1_0/tfar_client_state_1_0/
├── Cargo.toml  the `tfar_client_state_1_0` package (lib): tier 3, workspace lints and dependencies
└── src/        the session, the clients, the game data, the effects and the game link state
```

## How it works

A `SessionState` holds a `GameData` (what the game says about the local player: radios by
frequency, voice range, liveness, environment values, loudspeakers, the data frame, the local
push-to-talk state) and a `ClientDirectory` (one `ClientData` per TeamSpeak client, keyed by its
real nickname with the client id beside it). Each game or plugin message has an apply function on
`SessionState` (`apply_position`, `apply_frequencies`, `apply_killed`, `apply_speakers`,
`next_data_frame`, `apply_local_tangent`, `apply_local_release_all_tangents`,
`apply_remote_tangent`, `apply_remote_volume`, `apply_release_all_tangents`) that changes the
state and returns what the plugin app must do in TeamSpeak afterwards.

The local tangent (`TANGENT`, `TANGENT_LR`, `TANGENT_DD` from the game) acts only on a change of
the tangent flag: it writes the local player's transmit state (`prev_` + frequency after a
release), the half-duplex transmitting radio (set by a press that names its radio, cleared by a
release) and returns the local sound with its level from the short-range list for `digital`, else
the long-range list, and the broadcast; push-to-talk and VAD stay with the app. The game's
`RELEASE_ALL_TANGENTS` clears the flag and the transmit state, so the next press acts again.

A speaker's radio id and every vehicle descriptor (of a client and of a speaker) are held behind
an `Arc`, so a `HeardVia` names them with a new handle instead of a copy of the text; the shared
`local_radio_id` and `intercom_radio_id` stand for the carried radios and the intercom. A `POS`
replaces a client's vehicle only when it changed.

### Lock and ownership contract

- The plugin app keeps one `SessionState` per server connection behind one `parking_lot` mutex.
  Nothing here locks, spawns or calls TeamSpeak; time comes in as `now: Instant`.
- Every apply function returns owned values (`PositionApplied`, `RenameRequest`, `TangentEffects`,
  …). The app collects them under the lock, releases it, then makes the TeamSpeak calls. No
  reference into the state outlives the guard.
- The plugin-enabled check reads TeamSpeak metadata only when `needs_plugin_metadata` says the
  10 s cache has expired; the app reads it with the lock released and passes it to
  `plugin_enabled_for`.
- DSP objects belong to their sender's `ClientData::effects`. `ClientEffects::ensure_dsp` creates
  them off the audio thread (it allocates and designs filters); the audio thread borrows them by
  key through the `*_mut` accessors, which allocate nothing and answer `None` for an object never
  ensured, so the frame passes through.
- `GameLinkState` is one per plugin, beside the sessions; the game link worker records message
  times and the service loop polls it.

### Fixed original bugs

| Original bug | Test |
|---|---|
| shared state read outside its lock | the contract above: the state is plain data behind the app's lock |
| clients keyed by a nickname hash, first match on a collision | `nicknames_are_keyed_by_their_full_text_so_no_two_share_an_entry`, `a_rename_moves_the_entry_with_its_state` |
| direct speech and radio share one compressor | `direct_speech_and_radio_each_own_a_compressor` |
| a repeated PRESSED plays the end sound | `a_repeated_pressed_plays_no_sound_and_changes_no_count` |
| the positive plugin check never expires | `a_positive_plugin_check_expires_ten_seconds_after_the_metadata_was_read` |
| a liveness query invalidates the data frame | `is_alive_is_a_pure_query_and_never_changes_the_data_frame`, `the_sweep_expires_stale_clients_and_names_them` |
| proposed: the receive count drifts from its frequency set | `two_senders_on_one_frequency_each_count_and_each_release_ends_its_own` |
| proposed: `RELEASE_ALL_TANGENTS` leaves the reception counted | `release_all_tangents_stops_the_reception_it_ended` |
| proposed: a tangent resets the extrapolation clock | `a_tangent_press_keeps_the_position_extrapolation_clock` |
| proposed: one static object-interception strength for all clients | `every_client_redesigns_its_object_filters_when_the_strength_changes` |
| proposed: a 0 Hz or negative object-interception cutoff | `an_object_cutoff_at_or_below_zero_is_silence` |
| proposed: `rstflt` forgets the airborne effects | `resetting_radio_effects_drops_the_airborne_effects_too` |
| `RELEASE_ALL_TANGENTS` leaves the tangent pressed | `release_all_tangents_clears_the_tangent_so_the_next_press_acts` |

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_client_state_1_0  # the unit tests
```

## Configuration

- `pluginTimeout` (default 4 s, `tfar_plugin_settings_1_0`): read by `GameLinkState::poll` and
  `is_connected`.
- `objectInterceptionStrength` (default 400): the caller passes it in `DspContext`.

## Public surface

- `SessionState` and its apply functions; `GameData`, `Speaker`; `ClientDirectory`, `ClientData`.
- The reception vocabulary the routing crate fills: `HeardVia`, `SendingRadio`, `ReceivedOn`,
  `TangentRadio`, `IgnoreTangents`, `LOCAL_RADIO_ID`, `INTERCOM_RADIO_ID`, `local_radio_id`,
  `intercom_radio_id`.
- The results: `PositionApplied`, `RenameRequest`, `KilledApplied`, `VolumeApplied`,
  `ReleaseAllApplied`, `TangentEffects`, `TangentTransition`, `RemoteRadioSounds`,
  `RemoteSoundPlacement`, `LocalTangentApplied`, `LocalTangentSound`, `LocalReleaseAllApplied`,
  `RELEASED_FREQUENCY_PREFIX`.
- `ClientEffects`, `DspRequirement`, `DspContext`, `ObjectInterceptionFilter`, the effect keys,
  the filter designs `CANT_SPEAK_LOW_PASS`, `SPEAKER_BAND_PASS`, `PHONE_BAND_PASS`,
  `vehicle_low_pass`, and `object_interception_cutoff_hz`.
- `GameLinkState`, `GameLinkPoll`, `GameLinkTransition`; `DataFrame`; `radio_volume_gain`,
  `local_radio_sound`, `remote_radio_sound`; `ALIVE_TIMEOUT`, `PLUGIN_ENABLED_CACHE`, `TRANSITION_DEBOUNCE`.
- `prelude` also re-exports the foreign types in public fields (`StereoMode`, `RadioSubtype`,
  `FrequencyEntry`, `VehicleDescriptor`, `TeamspeakClientId`, `TangentBroadcast`, `TangentKind`).

## Boundaries

- Depends on: `tfar_geometry` (positions and directions); `tfar_arma_values` (frequency entries,
  stereo modes, subtypes); `tfar_game_commands_1_0` (the parsed game messages, the speaking code);
  `tfar_plugin_commands_1_0` (tangent broadcasts, client ids, the metadata test);
  `tfar_radio_effects_1_0` (the radio effects); `tfar_plugin_settings_1_0` (`pluginTimeout`);
  `tfar_dsp_filters` (the filters); `tfar_dynamics` (the compressors); `tfar_hrtf` (the HRTF
  sources); `tfar_sample_buffer` (the object-interception buffer); `thiserror`.
- Used by: `tfar_plugin_1_0`, `tfar_radio_routing_1_0`, `tfar_voice_pipeline_1_0`.
- Rules: effect state is only reached while its session's lock is held; clients are keyed by the
  full nickname (`nicknames_are_keyed_by_their_full_text_so_no_two_share_an_entry`); queries are
  pure (`is_alive_is_a_pure_query_and_never_changes_the_data_frame`). Tier 3 in
  `crates/version_1_0`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports, §5 for the data models and §8 for PTT and liveness.
- [Bug fixes](/documentation/program/bug_fixes.md) — the original bugs fixed here.
