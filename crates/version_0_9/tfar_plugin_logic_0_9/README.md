# Plugin logic 0.9

The safe logic of the TFAR 0.9.12 TeamSpeak plugin, one module per stage: `game_commands` parses
what the game sends, `plugin_commands` encodes what clients tell each other through TeamSpeak,
`client_state` holds every client's game state, `radio_routing` decides how the listener hears a
speaker, `radio_effects` shapes radio sound, and `voice_pipeline` runs it all on each voice frame.
`tfar_plugin_0_9` adds the TeamSpeak exports, threads and the game link server around it.

```bash
cargo test -p tfar_plugin_logic_0_9
```

Boundaries: depends on `tfar_foundation` and `tfar_audio`; used only by `tfar_plugin_0_9`. Never
depends on a 1.0 crate. `fastrand` stays in the two plugin logic crates
(`cargo xtask verify crate-tiers`, `dependency-firewalls`).

## `game_commands`

The `tfar_plugin_logic_0_9::game_commands` crate: the 0.9.12 messages Arma sends the plugin, parsed into typed
commands, and the reply strings the plugin answers with. Used by `tfar_plugin_logic_0_9::client_state`,
`tfar_plugin_0_9`.

### How it works

`GameCommand::parse` splits a message at tabs with `std::getline` semantics (a trailing empty field
is dropped), picks the parser by the first field and checks the field count the original checks.
Every number goes through `tfar_foundation::arma_values`; a malformed field is `Error::Malformed`, a wrong
count `Error::WrongFieldCount`, and both are answered `FAIL`. The crate has no state and calls no
TeamSpeak function: the plugin app does the side effect and picks the reply.

#### Replies

`reply_for_unhandled` gives the reply a command earns from its kind; for the three replies that
depend on TeamSpeak it gives the original's fallback, and the app answers with the looked-up value
when it has one.

| Message | Field count, name included | Reply |
|---|---|---|
| `TS_INFO SERVER` | exactly 2 | the server name; `ERROR_GETTING_SERVER_NAME` without one |
| `TS_INFO CHANNEL` | exactly 2 | the channel name; the empty string without one |
| `TS_INFO PING` | exactly 2 | `PONG` |
| `TS_INFO` other | exactly 2 | `FAIL` |
| `KILLED` | 2 or more | `DONE` |
| `SPEAKERS` | 1 or more | `OK` |
| `RELEASE_ALL_TANGENTS` | exactly 2 | `OK` (the original falls through to `FAIL`) |
| `VERSION` | 4, or 3 for an empty password (the original refuses 3) | `OK` |
| `TRACK` | exactly 2 | `OK` (the original falls through to `FAIL`) |
| `POS` | exactly 14 | `SPEAKING` when the player talks, else `NOT_SPEAKING` |
| `TANGENT`, `TANGENT_LR`, `TANGENT_DD` | 5, or 4 for an empty subtype (the original refuses 4) | `OK` |
| `FREQ` | exactly 14 | `OK` |
| `IS_SPEAKING` | exactly 2 | `SPEAKING` when the player talks, else `NOT_SPEAKING` |
| unknown name, wrong count, malformed field | — | `FAIL` |

#### Field rules

- `KILLED` accepts the 2 fields Arma sends; the original needs more than 2 and never runs.
- `SPEAKERS` reads records only from a message of exactly 2 fields; 1 field or more than 2 is an
  empty list, which clears the speakers.
- `VERSION` with an empty serious channel password loses its last field to the split; 3 fields
  read as an empty password, where the original refuses the handshake.
- A tangent from a radio without `tf_subtype` loses its empty last field the same way; 4 fields
  read as the empty, unknown subtype (no radio sound), where the original refuses the press.
- Booleans are `true` only for the exact text `true`; `POS` field 11 (`std::stoi`) and `FREQ`
  field 6 (`(int)atof`) truncate, the other integers round (`parseArmaNumberToInt`).
- `FREQ` field 11 (a second voice volume) is never read, so it is neither parsed nor checked.
- Game activity: every message but one starting with `VERSION` and longer than it counts
  (`message_counts_as_game_activity`, `GameCommand::counts_as_game_activity`).

```text
message ──split_fields(TAB)──▶ fields ──first field──▶ message_parsers::parse_* ──▶ GameCommand
                                                            │ field_values (numbers, "true", counts)
                                                            └─▶ Error (Malformed, WrongFieldCount, …)
GameCommand ──reply_for_unhandled──▶ &'static str        Error::reply ──▶ "FAIL"
```

`game_command.rs` splits the message with `tfar_foundation::arma_values::split_fields` and matches the first
field against `command_names.rs`; an unknown name is `GameCommand::Unknown` with the whole message.
Each parser in `message_parsers/` takes its fields by slice pattern, so a wrong count is an error
before any field is read, and reads its numbers through `field_values.rs`, which wraps
`tfar_foundation::arma_values`' errors with the command and field names. `vehicle_descriptor.rs` reads the
vehicle ids of `POS` and `SPEAKERS`. `replies.rs` maps a parsed command to the reply the original
gives it.

### Modules

#### `src/message_parsers/`

`GameCommand::parse` hands each parser the message's fields, the command name first. A parser
destructures them with a slice pattern of the exact count (`field_values::exact_fields`), so a
short or long message is `Error::WrongFieldCount` before a field is read; `KILLED` takes 2 or
more and `SPEAKERS` 1 or more. Numbers go through `field_values` (rounded like
`parseArmaNumberToInt`, truncated like `std::stoi`, or plain), booleans are the exact text `true`,
and the frequency lists, positions and vehicle ids go through `tfar_foundation::arma_values` and
`VehicleDescriptor`. Every failure names the command and the field.

### Configuration

None: the crate reads no setting.

### Public surface

- `GameCommand` and `GameCommand::parse`, `GameCommand::counts_as_game_activity`.
- The payload types `TsInfoQuery`, `SpeakerRecord`, `PositionUpdate`, `UsableRadios`,
  `TangentKind`, `FrequencyUpdate`, `VehicleDescriptor`.
- `replies` (`OK`, `FAIL`, `DONE`, `PONG`, `SPEAKING`, `NOT_SPEAKING`,
  `ERROR_GETTING_SERVER_NAME`, `NO_CHANNEL_NAME`) and `reply_for_unhandled`.
- `command_names`, `IN_GAME_TRIGGER_EXEMPT_PREFIX`, `message_counts_as_game_activity`.
- `Error` (with `Error::reply`), `Result`, `prelude`.

### Boundaries

- Depends on: `tfar_foundation::arma_values` (field splitting, numbers, arrays, frequency lists, radio
  subtypes); `tfar_foundation::geometry` (`Position`, `Angle`); `thiserror` for the error type.
- Used by: `tfar_plugin_logic_0_9::client_state`, `tfar_plugin_0_9`.
- Rules: the reply strings and field counts are those of `processGameCommand` in
  `reference/tfar_0_9_12/ts/src/plugin.cpp`, held by the tests in `src/tests/replies.rs` and
  `src/message_parsers/tests/`; every field access is length-checked, so a short message is an
  `Error`.

## `plugin_commands`

The `tfar_plugin_logic_0_9::plugin_commands` crate: the 0.9.12 plugin-to-plugin commands sent through TeamSpeak,
the sender filter and the client metadata block, byte-compatible with the original. Used by
`tfar_plugin_0_9`.

### How it works

Players' plugins talk through TeamSpeak's `sendPluginCommand` to the current channel, and show
their state in the `<TFAR>…</TFAR>` block of their client metadata. Both texts must stay byte for
byte those of `reference/tfar_0_9_12/ts/src/plugin.cpp`, so a Linux player and Windows players
with the original plugin read each other.

| Command | Text | Original |
|---|---|---|
| tangent | `TANGENT\|TANGENT_LR\|TANGENT_DD`, `PRESSED\|RELEASED`, frequency, range, subtype, nickname | plugin.cpp:1837, read at 3076 |
| release all | `RELEASE_ALL_TANGENTS`, nickname | plugin.cpp:1787, read at 3080 |
| volume | `VOLUME`, nickname, voice volume, `true\|false` | plugin.cpp:2906, read at 3084 |

Fields are tab-separated and split like `std::getline`. On decode a tangent's range is rounded and
the voice volume truncated as the original reads them; `PRESSED` and `true` are compared exactly.
The encoder writes the range as an integer, which every 0.9.12 decoder reads to the same value.
A command is taken only from a plugin whose name starts with `task_force_radio`
(`accepts_sender`, the `strncmp` of plugin.cpp:3116).

The status text is
`[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]Y[/B] [I]P:[/I][B]0.9.12[/B][I] A: [/I][B]<addon>[/B]`
(plugin.cpp:1300-1304), with `N` for a false flag; at shutdown it is
`[B]Task Force Radio Plugin Disabled[/B]`. Another client counts as playing when its block is
longer than `[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]Y` and starts with it (plugin.cpp:2531).
`merge_metadata` replaces the text of an existing block and keeps what surrounds it; without a
block the result is the block alone, as the original's `setMetaData` writes it.

```text
send:    PluginCommand ──encode──▶ text ──▶ sendPluginCommand(CURRENT_CHANNEL)
receive: onPluginCommandEvent(name, text) ──accepts_sender(name)──▶ decode(text) ──▶ PluginCommand
status:  format_metadata ──merge_metadata(own metadata)──▶ setClientSelfVariableAsString
others:  client metadata ──metadata_says_playing / info_data_text──▶ playing flag, info pane text
```

`plugin_command.rs` splits a received text with `tfar_foundation::arma_values::split_fields` and matches its
fields by slice pattern, so the field count is checked before any field is read; numbers go
through `tfar_foundation::arma_values`. `metadata.rs` finds the block from the first `<TFAR>` to the first
`</TFAR>` after it and never indexes: every cut is a checked `str::get`.

### Configuration

None: the crate reads no setting.

### Public surface

- `PluginCommand` with `encode` and `decode`, `TangentKind`, and the command names in
  `plugin_command`.
- `PLUGIN_NAME_PREFIX`, `accepts_sender`.
- `format_metadata`, `merge_metadata`, `metadata_block`, `metadata_says_playing`,
  `info_data_text`, `DISABLED_METADATA`, and the tags and sizes in `metadata`.
- `Error`, `Result`, `prelude`.

### Boundaries

- Depends on: `tfar_foundation::arma_values` (field splitting, numbers, `RadioSubtype`); `thiserror` for the
  error type.
- Used by: `tfar_plugin_0_9`.
- Rules: the encoded texts match what the original 0.9.12 plugin sends, held by the byte-exact
  tests in `src/tests/plugin_command.rs` and `src/tests/metadata.rs`.

## `client_state`

The `tfar_plugin_logic_0_9::client_state` crate: the per-client state of the 0.9.12 plugin: positions, radios,
tangents and effect state, with the application of every game message and plugin command. Used by
`tfar_plugin_0_9`, `tfar_plugin_logic_0_9::radio_routing`, `tfar_plugin_logic_0_9::voice_pipeline`.

### How it works

The crate is pure state and decisions: it calls no TeamSpeak function, starts no thread and reads
no clock (every time comes in as `now: Instant`). The plugin app owns one `ServerRadioData` per
TeamSpeak server connection behind one `parking_lot::Mutex`, applies each parsed game message
(`tfar_plugin_logic_0_9::game_commands`) and each decoded plugin command to it under the lock, and turns what the
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

#### Lock and ownership contract

- One lock per server connection guards the whole `ServerRadioData`, sound state included; no
  reference into it outlives the guard.
- Sound state is made only by `ClientData::ensure_dsp`, which the plugin calls while handling a
  command, with the `DspRequirements` that `tfar_plugin_logic_0_9::radio_routing::dsp_requirements` computes for
  the sender. The audio thread waits for the lock, held only for short decisions, and only
  borrows what exists through the `_mut` accessors; it never makes or drops a filter, effect or
  HRTF source.
- `reset_radio_effects` drops sound state and `reset_voices` clears filter memory, both on the
  command threads; `remove_expired` hands the expired entries back, so the service thread frees
  their sound state after releasing the server lock.
- `GameSession` is one per plugin, beside the server states, as the original's global `inGame`.

#### Fixed behaviour

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

#### Shared ids

A loudspeaker radio's id and every vehicle descriptor (of a `ClientData` and of a `SpeakerRadio`)
are held behind an `Arc`, so a reception of `tfar_plugin_logic_0_9::radio_routing` names them with a new handle
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

### Modules

#### `src/command_application/`

Each method takes the typed command (`tfar_plugin_logic_0_9::game_commands` for the game, the decoded fields of
`tfar_plugin_logic_0_9::plugin_commands` for the plugins) and `now` where the original stamps a time, changes the
state as `processGameCommand`, `processUnitPosition` and `processPluginCommand` in
`reference/tfar_0_9_12/ts/src/plugin.cpp` do, and returns an outcome naming the TeamSpeak side
effect: `PositionOutcome` (centre the listener, check the mute, or refresh the client list),
`RenameRequest`, the tangent-changed flag, `RemoteTangentOutcome` (route, mute and play the start or
end sound) and `RemoteVolumeOutcome`. None of them fails: an unknown nickname changes nothing and
says so.

#### `src/dsp_state/`

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
direction sources restart themselves after 200 ms without input (`tfar_audio::hrtf`); the compressor stays.

### Configuration

None: the crate reads no setting. The HRIR sphere reaches it as an optional `&HrirLibrary` in
`ensure_dsp`; without one no direction source is made.

### Public surface

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

### Boundaries

- Depends on: `tfar_foundation::geometry` (positions and angles); `tfar_plugin_logic_0_9::game_commands` (the typed game
  messages, `TangentKind`, `VehicleDescriptor`); `tfar_plugin_logic_0_9::radio_effects` (the radio effects);
  `tfar_audio::dsp_filters` (the Butterworth filters); `tfar_audio::dynamics` (the compressor); `tfar_audio::hrtf` (the
  direction sources); `thiserror` for the error type.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_logic_0_9::radio_routing`, `tfar_plugin_logic_0_9::voice_pipeline`.
- Rules: no TeamSpeak call, thread or clock; sound state is made only off the audio thread
  (`src/dsp_state/tests/client_dsp.rs`); the fixed behaviours above are pinned by
  `in_game_without_a_serious_channel`, `the_original_nickname_after_two_renames`,
  `a_lookup_by_client_id_returns_the_fresh_entry_not_a_stale_one_left_by_a_rename` and
  `fields_the_original_leaves_uninitialised_start_at_defined_defaults`.

## `radio_routing`

The `tfar_plugin_logic_0_9::radio_routing` crate: whether and how a 0.9.12 listener hears a speaker: direct, over
radio or over a speaker radio, and which players the plugin mutes. Used by `tfar_plugin_0_9`,
`tfar_plugin_logic_0_9::voice_pipeline`.

### How it works

Every function reads a borrowed `tfar_plugin_logic_0_9::client_state::ServerRadioData` under the plugin's lock
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

### Modules

#### `src/mute_decisions/`

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

#### `src/reception/`

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

### Configuration

None: the crate reads no setting.

### Public surface

- `is_over_local_radio`, `is_over_radio`, `is_over_radio_into`, `receptions`, `HeardVia`,
  `HeardOn`, `IgnoreTangents`, `LOCAL_RADIO_ID`, `local_radio_id`.
- `raw_distance`, `effective_distance`, `diver_radio_distance`, `effect_error_from_distance` and
  the diver reach constants.
- `MuteTracker`, `MuteDecision`, `MuteQuery`, `GameClientMute`, `game_client_mute`,
  `set_game_client_mute_status`, `set_mute_for_dead_players`.
- `is_serious_mode`, `is_other_radio_plugin_channel`, `OTHER_RADIO_PLUGIN_CHANNELS`.
- `dsp_requirements`, `Error`, `Result`, `prelude`.

### Boundaries

- Depends on: `tfar_plugin_logic_0_9::client_state` (the state it reads and the sound requests it fills);
  `tfar_foundation::geometry` (positions); `thiserror` for the error type.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_logic_0_9::voice_pipeline`.
- Rules: the check order and the strict range comparisons of the original
  (`src/reception/tests/local_radio.rs`); an unmute only for a client this plugin muted
  (`outside_serious_mode_unmute_goes_only_to_clients_this_plugin_muted`); a missing entry is muted
  without touching voice state (`a_missing_entry_is_muted_without_touching_voice_state`); no
  function but the two `set_*` ones changes state.

## `radio_effects`

The `tfar_plugin_logic_0_9::radio_effects` crate: the 0.9.12 radio sound chains: filters and noise per radio type.
Used by `tfar_plugin_logic_0_9::client_state`, `tfar_plugin_logic_0_9::voice_pipeline`.

### How it works

The crate ports `reference/tfar_0_9_12/ts/src/RadioEffect.h` and the `processRadioEffect` mixdown of
`reference/tfar_0_9_12/ts/src/plugin.cpp:2476`. A voice pipeline owns one effect per sender and radio:
[SW](/documentation/glossary.md) radios use `PersonalRadioEffect`, [LR](/documentation/glossary.md)
and airborne radios `LongRangeRadioEffect`, and [DD](/documentation/glossary.md) radios carried under
water `UnderWaterRadioEffect`. It sets the error level from the distance, then hands
`process_radio_effect` the block copied for that radio:

```text
stereo i16 block --mono mixdown, /32767, x gain--> mono f32 block --effect--> clip, x32766 --> one or both ears
```

The two speaker effects run, per sample, a 2399-sample delay, ring modulation by the error level,
foldback distortion, an [RBJ](/documentation/glossary.md) high-pass and low-pass and ×30; the
personal effect filters its microphone band first. The underwater effect drops samples at random
with about the error level as probability, band-passes round 1000 Hz and multiplies by 30. Effects
are built off the TeamSpeak audio thread; processing runs on it and allocates nothing.

`process_radio_effect` mixes each stereo frame to `trunc((left + right) / 2)`, scales it to
`sample / 32767 · gain` (×1.5 for one ear) in the `MonoMixBuffer`, calls `RadioEffect::process` on
the whole block, and writes `trunc(sample · 32766)` back, 32767 above 1 and −32768 below −1, to one or
both channels. The effects in `effects/` are built from `signal_stages/` and the `MonoFilter` of
`tfar_audio::dsp_filters`; the speaker effects map their error level through `error_level.rs`, the
underwater effect uses the raw value. Each filter output is rounded back to `f32`, as the
original's float buffers are. `tests/signal_measurements.rs` holds the sines, RMS, tone share and
band-shape helpers the effect tests share.

### Modules

#### `src/effects/`

`speaker_chain.rs` computes the foldback threshold `0.3 · (1 − error) · mean(|x|) / 0.005` from the
block as it arrives, then runs each sample through the delay line, the ring modulator, the foldback,
the RBJ high-pass, the RBJ low-pass and ×30. `long_range.rs` gives it its filters; `personal.rs`
first runs its microphone high-pass and low-pass over the whole block, so the threshold comes from
the microphone-filtered signal. `under_water.rs` draws one dropout decision per sample, then
band-passes and multiplies by 30. 0.9.12 has no airborne effect: airborne radios use the long-range
one.

#### `src/signal_stages/`

Each stage is a small value with the state the original keeps in its effect object: the delay
line's slots and position, the ring modulator's phase, the dropout's generator. The stages hold no
filters; the effects in `effects/` add those. The dropouts replace the original's global, unseeded
`rand()` with one `fastrand` generator per effect, seeded by its creator, so runs are reproducible
and two effects never share a sequence; the draw keeps the 0..=32767 range of the Windows `rand()`.

### Configuration

None: the crate reads no setting. The underwater dropouts draw from a generator the creator seeds.

### Public surface

- `RadioEffect`: `set_error_level` and `process`, implemented by `LongRangeRadioEffect`,
  `PersonalRadioEffect` and `UnderWaterRadioEffect`; `SAMPLE_RATE_HZ`.
- `error_level_from_distance`: the 0.9.12 error-level curve.
- `process_radio_effect` and its reusable `MonoMixBuffer`.
- `Error`, `Result` and `prelude` (which also re-exports `StereoMode`).

### Boundaries

- Depends on: `tfar_audio::dsp_filters` (the Butterworth and RBJ filters); `tfar_audio::sample_buffer` (the
  stereo block); `tfar_foundation::arma_values` (`StereoMode`); `fastrand` for the seeded dropouts;
  `thiserror` for the error type.
- Used by: `tfar_plugin_logic_0_9::client_state`, `tfar_plugin_logic_0_9::voice_pipeline`.
- Rules: the error level rises with distance, never falls, and stays inside its table
  (`src/tests/error_level.rs`); each effect keeps the original's processing order and filters
  (the tests in `src/effects/tests/`).

## `voice_pipeline`

The `tfar_plugin_logic_0_9::voice_pipeline` crate: the 0.9.12 stereo function per sender, the path one
TeamSpeak voice frame takes through the gates, the direct voice, the radio effects, the
loudspeakers, the HRTF and the mix, plus the local player's own voice for loudspeakers and the
positional remote push-to-talk sound. Used by `tfar_plugin_0_9`.

### How it works

The plugin owns one `VoiceProcessor` per server connection beside its `ServerRadioData`, both
behind the connection's lock. In the post-process callback it copies the frame to stereo
(`tfar_audio::sample_buffer::copy_to_stereo`), reads what only TeamSpeak knows into a
`VoiceFrameContext` (the sender's and the listener's client ids, whether the sender's metadata
shows the plugin, the sender's channel name, the in-game flag), waits for the state lock, calls
`process_sender`, and writes the frame back (`write_back_from_stereo`). The context carries client
ids, not a nickname, because the original gates on TeamSpeak's client id (`hasClientData`,
`reference/tfar_0_9_12/ts/src/plugin.cpp:584`); the metadata and the channel name come in as values
so the processor makes no TeamSpeak call under the lock.

#### The gates

`process_sender` follows `ts3plugin_onEditPostProcessVoiceDataEventStereo`
(`reference/tfar_0_9_12/ts/src/plugin.cpp:2653-2828`) and answers `VoiceOutcome`:

- sender and listener have current data and the plugin is enabled for the sender
  (`ClientData::refresh_plugin_enabled`, with its 10 s grace): `Silenced` when the sender sits in
  the [serious mode](/documentation/glossary.md#serious-mode) channel and the listener is dead,
  `Processed` otherwise;
- else the local player's own frame is `Silenced`; an alive, in-game listener hears a serious-channel
  TFAR player without current data as `Silenced`, unless the channel is another radio plugin's;
- everyone else is `PassThrough`, untouched. The original's "no info" debug line, every 4 s, is
  left to the plugin, which sees the `PassThrough`.

#### The chain

Each frame is planned first (`src/voice_chain/sender_plan.rs`): with the state borrowed read-only it
computes the direct voice's branch and level, asks `tfar_plugin_logic_0_9::radio_routing::is_over_radio_into` for
the receptions in the scratch reception list and plans each one's effect gain, effect error and
loudspeaker stages into the scratch route list. Then the sender's entry is borrowed mutably and the frame renders over its `ClientDsp`
(`src/voice_chain/frame_render.rs`), in the original's order:

```text
frame ── × voice volume multiplier ──┬─▶ direct voice (in range, not me) ── level ── direction ─┐
                                     │   else silence                                           │
                                     └─▶ copy ── compressor (once) ──┬─ reception 1 ─▶ mix ◀────┤
                                                                     └─ reception n ─▶ mix ◀────┘
                                                                      ── × global volume ── out
```

- **Distance gain:** `volume_from_distance` (`volumeFromDistance`, plugin.cpp:698):
  `d^-0.3 · max(0, reach − d) / reach`, 1 within a metre, 0 below 0.001, at most 1; the reach is the
  sender's voice volume times the vehicle multiplier, or 5 m when either side cannot speak. A reach
  of zero or less gives 0, where the original divides by zero.
- **Direct voice:** both can speak and no vehicle muffles: the distance gain; a vehicle muffles
  (combined isolation of at least 0.01, different vehicles): the vehicle low-pass at
  `20000 · (1 − isolation) / 4` Hz with the gain `volume(d, reach · (1 − isolation)) ·
  (1 − isolation)^1.2`; either side cannot speak: the 100 Hz low-pass at ×14.
- **Radio:** one compressor pass over the copy (threshold 65 dB, ratio 0.1, 1 ms / 300 ms), then per
  reception, by the transmitter's subtype: `digital` the personal effect, `digital_lr` and
  `airborne` the long-range effect, both at `((v + 1) / 10)^4 · 0.35` with the error from the
  effective distance over the range; `dd` the underwater effect at the diver volume level `· 0.6`
  with the error from the raw distance over the diver reach; `phone` the phone band-pass at
  `· 10`; any other subtype nothing. The effect writes to both ears or one, as the receiving radio
  is set.
- **Loudspeaker:** the 2000/1000 Hz band-pass at ×4; the distance gain over the loudspeaker's reach
  (`volume / 10 · speaker distance`, truncated to whole metres, 5 m when the sender cannot speak), or
  the vehicle low-pass when a vehicle stands between; below the wave line the 100 Hz low-pass at ×14;
  then the direction of the loudspeaker.
- **Mix and output:** each reception is added with saturation; the global volume is the last gain.

#### Choices where the port differs from the original

- **Direction (one panner).** The original runs clunk (an ear delay only) and then the simple ILD on
  the direct voice and on every loudspeaker. Here a `tfar_audio::hrtf::HrtfSource`, the one the sound state
  holds per sender (direct voice) and per radio id (loudspeaker), places the sound from the
  listener's view angle, and no ILD follows. The ILD with the 0.9 coefficient 0.375 is kept only as
  the fallback for a sound without an HRTF source: no HRIR sphere was loaded, or the source is not
  made yet.
- **Direction after the level.** The original places the sound before the level stage; here the
  level stage (gain or filter) runs first and the HRTF last. Both are linear, so the order changes
  only where a stage clips.
- **HRTF makeup gain.** The IRCAM sphere is diffuse-field equalised and puts a source straight ahead
  about 4 dB below its input; `HRTF_MAKEUP_GAIN_DB` (+4 dB) goes into the HRTF stage of direct speech
  so speech straight ahead keeps the original's level. Sources at the side and behind stay quieter,
  as the sphere measures them. A loudspeaker's HRTF stage runs at unity, as the 1.0 pipeline renders
  its speakers: the simple ILD it replaces already puts a loudspeaker straight ahead at 0.625, about
  the sphere's front level (`a_loudspeaker_through_the_hrtf_gets_no_makeup_gain`). The ILD fallback
  gets no makeup.
- **Sound state is borrowed, never made.** The original makes every filter, effect and ear-delay
  stage on first use on the audio thread. Here they come from `ClientData::ensure_dsp`, which the
  plugin calls while handling commands with `tfar_plugin_logic_0_9::radio_routing::dsp_requirements`. A path whose
  object is missing (a command not handled yet) stays silent for that frame, never unfiltered (a
  can't-speak voice without its low-pass would play at ×14), and `frames_missing_sound_state` counts
  the frame.
- **Plugin-enabled cache entry.** The cache is the sender's current entry, else any entry with the
  sender's client id, where the original takes the first entry by nickname order.
- **Saturation.** Every gain and the mix saturate at the `i16` limits (`tfar_audio::sample_buffer`).

#### Own voice

`process_own_voice_for_speakers` runs the local player's captured frame through the same chain with
the local player as sender (`ts3plugin_onEditCapturedVoiceDataEvent`, plugin.cpp:2852): their
direct voice is silent and their own radios are skipped, so only a loudspeaker of another player on
the frequency they transmit on sounds. It returns nothing outside a game, for a dead local player or
without current data; `append_own_voice` appends the block to the playback mixer's
`my_radio_voice` stream off the audio thread only (it takes the stream lock); the plugin's
captured-voice callback pushes the block through a `StreamProducer` for that stream instead. The
captured frame itself is never changed.

#### Memory on the audio path

`VoiceScratch` holds every buffer the chain writes, sized once (`DEFAULT_MAX_FRAMES`, 100 ms) and
reused, so a frame within that size changes no capacity. That includes the reception list:
`tfar_plugin_logic_0_9::radio_routing::is_over_radio_into` refills it in place, and each reception shares its
radio id and vehicle with the server state (`Arc`), so routing copies no text. The reception and
route lists are emptied at the end of each frame, still under the server lock, so the audio thread
never drops the last handle on a radio id and never frees one.

#### Remote push-to-talk sound

`render_remote_tangent_sound` shapes a decoded `remote_start` or `remote_end` WAV as the positional
`playWavFile` does (plugin.cpp:711-785): the radio volume and global gain, then, with current
listener data, the vehicle low-pass, on a loudspeaker the distance gain, the speaker band-pass and
the under-water low-pass at `CANT_SPEAK_GAIN * 50`, and last the direction through a one-shot HRTF
source at the makeup gain (its latency cut off) or the simple ILD without a sphere. The
`RemoteSoundPlan` says what to apply. It builds filters and an HRTF source per sound, so the
plugin calls it while handling the incoming command, never on the audio thread.

```text
plugin ── VoiceFrameContext, &mut ServerRadioData, &mut StereoBuffer ──▶ VoiceProcessor
   voice_processor: gates ──▶ PassThrough | Silenced | run the chain
   voice_chain: plan_sender (read-only) ──▶ SenderPlan + routes in VoiceScratch
                render_frame (sender's ClientDsp) ──▶ Processed
own_voice: captured frame ──▶ same chain as the local player ──▶ block ──▶ PlaybackMixer stream
remote_tangent_sound: WAV + RemoteSoundPlan (command thread) ──▶ Playback ──▶ PlaybackMixer
```

`voice_processor.rs` holds the gates of the stereo function and the plugin-enabled check, and runs
the chain in two steps so that the read-only plan and the mutable render never borrow the state
together; the sender's nickname and the receptions travel between them in `VoiceScratch`.
`distance_gain.rs` and `voice_gains.rs` hold the numbers the plan uses. `own_voice.rs` adds the
captured-voice path to the processor. `remote_tangent_sound.rs` shapes a remote push-to-talk sound
off the audio thread with the same gains and filter designs. The tests share one synthetic scene
(`tests/test_scene.rs`, compiled only for tests).

### Modules

#### `src/voice_chain/`

`sender_plan.rs` reads the server state once: the direct voice's branch and level, and per
reception from `tfar_plugin_logic_0_9::radio_routing::is_over_radio_into` (into the scratch reception list) its
effect gain, its effect error and its loudspeaker stages, each with a `SoundPlacement` (the HRTF
direction from the listener's view angle and the 0.9 ILD gains). `frame_render.rs` then runs the
plan over the sender's `ClientDsp`: `direct_voice.rs` for the voice in range, one compressor pass
over the copy, and `radio_reception.rs` for each reception on its own copy, mixed in with
saturation. `sound_placement.rs` is the one panner both use: the direct voice renders through the
HRTF at the makeup gain, a loudspeaker at unity. A step whose sound object is not made yet reports
it and contributes silence.

### Configuration

None: the crate reads no setting. The levels it uses (global volume, diver volume level, loudspeaker
distance, terrain coefficient) arrive in `ServerRadioData` from the game's `FREQ` message.

### Public surface

- `VoiceProcessor` with `new`, `process_sender`, `process_own_voice_for_speakers`, `scratch` and
  `frames_missing_sound_state`; used by `tfar_plugin_0_9` from the voice callbacks.
- `VoiceFrameContext`, `OwnVoiceContext`, `VoiceOutcome`.
- `VoiceScratch`, `ScratchCapacities`, `DEFAULT_MAX_FRAMES`, `ROUTE_CAPACITY`, `NICKNAME_CAPACITY`.
- `append_own_voice`, `OWN_VOICE_STREAM_NAME`.
- `render_remote_tangent_sound`, `RemoteSoundPlan`, `ListenerView`,
  `UNDER_WATER_SOUND_GAIN_FACTOR`; used by `tfar_plugin_0_9` for an incoming push-to-talk.
- `volume_from_distance`, `radio_volume_multiplier`, `CANT_SPEAK_DISTANCE`, `MINIMUM_AUDIBLE_GAIN`, also the
  volume curve of the remote push-to-talk sounds.
- The gain constants `CANT_SPEAK_GAIN`, `SPEAKER_GAIN`, `RADIO_EFFECT_GAIN`,
  `DIVER_RADIO_EFFECT_GAIN`, `PHONE_GAIN`, `VEHICLE_LOSS_EXPONENT`, `HRTF_MAKEUP_GAIN_DB`, with
  `hrtf_makeup_gain` and `vehicle_loss_gain`.
- `Error`, `Result`, `prelude`.

### Boundaries

- Depends on: `tfar_plugin_logic_0_9::client_state` (the state and the sound state it borrows);
  `tfar_plugin_logic_0_9::radio_routing` (the receptions, distances and serious-mode checks);
  `tfar_plugin_logic_0_9::radio_effects` (`process_radio_effect` and the effects); `tfar_audio::dsp_filters` (the filters
  the sound state holds); `tfar_audio::dynamics` (the compressor); `tfar_audio::hrtf` (the HRTF sources and the
  listener-relative direction); `tfar_audio::spatial_audio` (the 0.9 ILD fallback); `tfar_audio::sample_buffer`
  (the stereo buffer, saturating gain and mix); `tfar_audio::playback_mixer` (the own-voice stream and the
  remote sound's playback); `thiserror` for the error type; `tracing` for a refused remote sound
  filter.
- Used by: `tfar_plugin_0_9`.
- Rules: no TeamSpeak call and no sound object made or dropped (the processor only borrows
  `ClientDsp`); no scratch capacity changes for frames within the scratch's size
  (`a_thousand_frames_change_no_scratch_capacity`, which also holds the reception list empty and the
  shared radio ids unchanged between frames); the compressor runs once per frame
  (`the_compressor_envelope_advances_once_per_frame_whatever_the_receptions`); a missing sound object
  silences its path (`a_frame_missing_a_needed_filter_silences_that_path_and_is_counted`).
