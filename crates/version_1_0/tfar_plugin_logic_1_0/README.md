# Plugin logic 1.0

The safe logic of the TFAR 1.0 TeamSpeak plugin, one module per stage: `plugin_settings` holds what
`SETCFG` sets, `antennas` models the radio towers, `game_commands` parses what the game sends,
`plugin_commands` encodes what clients tell each other through TeamSpeak, `client_state` holds every
client's game state, `radio_routing` decides how the listener hears a speaker, `radio_effects`
shapes radio sound, and `voice_pipeline` runs it all on each voice frame. `tfar_plugin_1_0` adds the
TeamSpeak exports, threads and the game link server around it.

```bash
cargo test -p tfar_plugin_logic_1_0
```

Boundaries: depends on `tfar_foundation` and `tfar_audio`; used only by `tfar_plugin_1_0`. Never
depends on a 0.9 crate. `fastrand` stays in the two plugin logic crates
(`cargo xtask verify crate-tiers`, `dependency-firewalls`).

## `plugin_settings`

The `tfar_plugin_logic_1_0::plugin_settings` crate: the 1.0 plugin's settings: the values `SETCFG` sets and the
defaults. Used by `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_logic_1_0::game_commands`, `tfar_plugin_1_0`,
`tfar_plugin_logic_1_0::radio_routing`, `tfar_plugin_logic_1_0::voice_pipeline`.

### How it works

`PluginSettings` holds the twenty settings of `reference/tfar_1_0/ts/src/settings.hpp` as typed
fields, each starting at its default. `set(key_text, value_text, type_tag)` applies one `SETCFG`
message as `CommandProcessor.cpp:388-414` does:

- **Key:** the first setting, in the original's order, whose wire name equals the start of the key
  text in ASCII case-insensitive comparison (`_strnicmp` over the name's length, `settings.hpp:47`).
  The prefix rule is kept so every key the addon sends still matches; no wire name is a prefix of
  another, so the first match is the only one.
- **Value:** `BOOL` is true only for `true` or `TRUE`, `SCALAR` is a finite number, any other tag
  is text. `None` stands for a message whose token count is not 4, where the original sets the
  empty text.
- **Conversion:** the value takes the kind of the setting's default: a boolean from a number is
  `number > 0`, a number from a boolean 1 or 0, a text from a boolean `true`/`false` and from a
  number the six-decimal `%f` text. A text set on a boolean or number setting is an error (the
  original writes it through the wrong union member).

Every error leaves the settings unchanged. The refresh flag behind `NEEDCFG` starts true, clears on
any successful set and is set again by `mark_needs_refresh` when the game disconnects.

`PluginSettings::set` finds the key with `SettingKey::from_key_text`, reads the value with
`TypeTag::read_value` (or takes the empty text when there is no tag), and converts it into the
setting's kind with `SettingValue::into_bool`, `into_number` or `into_text` before writing the
typed field; only then does the refresh flag clear. `SettingKey::default_value` is the single
table of defaults and kinds; `PluginSettings::default` holds the same values in its fields, and a
test holds the two equal.

### Configuration

The crate reads no setting itself; it holds the plugin's. The twenty settings, their defaults and
getters (`settings.hpp:12-31`):

| Key | Kind | Default | Getter |
|---|---|---|---|
| `full_duplex` | bool | true | `full_duplex()` |
| `addon_version` | text | `unknown` | `addon_version()` |
| `serious_channelName` | text | empty | `serious_channel_name()` |
| `serious_channelPassword` | text | empty | `serious_channel_password()` |
| `intercomVolume` | number | 0.1 | `intercom_volume()` |
| `intercomEnabled` | bool | true | `intercom_enabled()` |
| `pluginTimeout` | number (s) | 4 | `plugin_timeout() -> Duration` |
| `headsetLowered` | bool | false | `headset_lowered()` |
| `spectatorNotHearEnemies` | bool | false | `spectator_not_hear_enemies()` |
| `spectatorCanHearFriendlies` | bool | true | `spectator_can_hear_friendlies()` |
| `tangentReleaseDelay` | number (ms) | 0 | `tangent_release_delay() -> Duration` |
| `moveWhileTabbedOut` | bool | false | `move_while_tabbed_out()` |
| `intercomDucking` | number | 0.2 | `intercom_ducking()` |
| `minimumPluginVersion` | number | 0 | `minimum_plugin_version()` |
| `objectInterceptionStrength` | number | 400 | `object_interception_strength()` |
| `voiceCone` | bool | true | `voice_cone()` |
| `allowDebugging` | bool | true | `allow_debugging()` |
| `noAutomoveSpectator` | bool | false | `no_automove_spectator()` |
| `disableAutomaticMute` | bool | false | `disable_automatic_mute()` |
| `muteSpectators` | bool | false | `mute_spectators()` |

The two durations are cut to whole milliseconds and never negative.

### Public surface

- `PluginSettings`: `default()`, `set`, `value`, `needs_refresh`, `mark_needs_refresh` and one getter
  per setting.
- `SettingChanged`: the key a successful set wrote.
- `SettingKey`: `ALL`, `COUNT`, `wire_name`, `default_value`, `kind`, `from_key_text`.
- `SettingValue`, `SettingKind`: a value, its kind and the cross-kind conversions.
- `TypeTag`: `from_wire`, `read_value`.
- `Error` (`UnknownKey` with the original's message, `MalformedNumber`, `TextForTypedSetting`),
  `Result`, `prelude`.

### Boundaries

- Depends on: no workspace crate; `thiserror` for the error type.
- Used by: `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_logic_1_0::game_commands`, `tfar_plugin_1_0`,
  `tfar_plugin_logic_1_0::radio_routing`, `tfar_plugin_logic_1_0::voice_pipeline`.
- Rules: an unknown key or a malformed value is an `Error` and leaves the settings unchanged
  (`a_failed_set_changes_nothing_and_keeps_the_refresh_flag`); the defaults and key order are the
  original's (`the_defaults_are_the_original_table`).

## `antennas`

The `tfar_plugin_logic_1_0::antennas` crate: the 1.0 antenna and radio tower model used for terrain-aware
reception. Used by `tfar_plugin_1_0`, `tfar_plugin_logic_1_0::radio_routing`.

### How it works

The game reports radio towers with `RadioTwrAdd` (`netId;range;[x,y,z]` records) and removes them
with `RadioTwrDel`. The plugin app turns each parsed record into an `Antenna` and hands it to
`AntennaManager::add`, which replaces a tower of the same `NetId` in place and ignores the null
id; `remove` drops towers by id.

For every sender, the routing crate asks `find_connection(sender, sender_range, receiver)`, which
follows `AntennaManager::findConnection` (`reference/tfar_1_0/ts/src/antennaManager.cpp:24-46`):
the candidates are the towers the sender reaches (strictly closer than its range) and that reach
the receiver (strictly closer than the tower's range); the loss of a candidate is
`d(receiver)/tower_range + d(sender)/sender_range`; the lowest loss below 1 wins, the first
tower on a tie.

The original returns a connection with loss 7 for "none". Here "none" is `None`. Callers that
take the minimum of a distance error and the antenna loss, as the voice pipeline does for a
radio effect's error level, map `None` to `NO_CONNECTION_LOSS` (7) with
`loss_or_no_connection`, which keeps the original's cap exactly.

`NetId::parse` reads `creator:object` within its own text: the original runs `strchr` over a
`string_view` that is not terminated, so an id without a colon borrows the colon of the text
after it (`reference/tfar_1_0/ts/src/datatypes.hpp:187-212`).

`AntennaManager` keeps a `Vec<Antenna>` in insertion order; `find_connection` scans it once,
keeping the tower of lowest loss below 1 among those both ends reach, and returns it with its loss
as an `AntennaConnection` (a copy of the tower, so no reference outlives the caller's lock).
`Antenna::new` refuses a negative or non-finite range, which the original stores but can never
connect through.

### Configuration

None: the crate reads no setting.

### Public surface

- `Antenna`: a tower (`new(id, position, range)`, `is_reached_by`, `reaches`, `connection_loss`).
- `AntennaManager`: `add`, `remove`, `clear`, `antennas`, `find_connection`.
- `AntennaConnection` (`antenna`, `loss`), `NO_CONNECTION_LOSS`, `loss_or_no_connection`.
- `NetId`: `new`, `parse`, `creator`, `object`, `is_null`, `NULL`.
- `Error`, `Result`, `prelude`.

### Boundaries

- Depends on: `tfar_foundation::geometry` (positions and distances); `tfar_foundation::arma_values` (the integer reader
  of `NetId::parse`); `thiserror` for the error type.
- Used by: `tfar_plugin_1_0`, `tfar_plugin_logic_1_0::radio_routing`.
- Rules: a connection is only found through towers both ends reach, with a loss below 1
  (`the_lowest_loss_tower_below_one_is_chosen`, `a_tower_the_sender_cannot_reach_is_no_connection`);
  the net id is read within its own text (`net_id_parse_never_reads_past_its_own_text`).

## `game_commands`

The `tfar_plugin_logic_1_0::game_commands` crate: the 1.0 messages Arma sends the plugin, parsed into typed
commands, and the answers the plugin gives sync calls. Used by `tfar_plugin_logic_1_0::client_state`,
`tfar_plugin_1_0`.

### How it works

`parse(text)` splits a message at its tabs, keeping every field as `helpers::split` does
(`reference/tfar_1_0/ts/src/helpers.cpp:245-262`; a trailing tab gives a trailing empty field),
finds the command by exact name in place of the original's FNV-1a hash switch, and reads its
fields by position with checked counts. The text is the message without the async `~`, which the
game link removes. An unknown name is `GameCommand::Unknown`; a known command with a wrong count
or a malformed field is an `Error` that rejects the whole message, never an out-of-bounds read or
a silent zero.

#### Token counts

| Command | Tokens, name included | Notes |
|---|---|---|
| `TS_INFO` | 2 or more | only the second is read, as the sync split stops there |
| `POS` | exactly 14 | |
| `IS_SPEAKING` | 2 or more | only the second is read |
| `IS_SPEAKING_BULK` | any | every token after the name is a nickname, a trailing empty one too |
| `FREQ` | exactly 11 | |
| `KILLED`, `RELEASE_ALL_TANGENTS` | exactly 2 | |
| `TANGENT`, `TANGENT_LR`, `TANGENT_DD` | 5, or 6 with the class name | |
| `SETCFG` | 3 or more | exactly 4 carries the type tag; any other count sets the empty text |
| `SPEAKERS` | any | any count but 2 is the empty list, which clears the speakers |
| `RadioTwrAdd`, `RadioTwrDel` | exactly 2 | |
| `TRACK` | exactly 4 | |
| `DFRAME`, `MISSIONEND`, `collectDebugInfo` | any | no field is read |

`RECV_FREQS` is not a command: the original never maps it.

#### Sync replies

`GameCommand::is_async_only` holds the original's rule: `TS_INFO`, `POS`, `IS_SPEAKING` and
`IS_SPEAKING_BULK` answer sync (`POS` is also queued); every other command answers
`ASYNC COMMAND SENT IN SYNC CONTEXT` when called sync.

| Situation | Reply | Source (`reference/tfar_1_0/ts/src/`) |
|---|---|---|
| unknown name | `UNKNOWN COMMAND` | `CommandProcessor.cpp:80` |
| async-only command called sync | `ASYNC COMMAND SENT IN SYNC CONTEXT` | `CommandProcessor.cpp:161` |
| `TS_INFO PING` | `PONG` | `CommandProcessor.cpp:625` |
| `TS_INFO VERSION` | the plugin version | `CommandProcessor.cpp:627` |
| `TS_INFO SERVER`, `SERVERUID` | the value, or `ERROR_GETTING_SERVER_NAME` | `Teamspeak.cpp:550-574` |
| `TS_INFO CHANNEL` | the channel name, or the empty text | `Teamspeak.cpp:526-539` |
| `TS_INFO CHANNELID` | the channel id in decimal | `CommandProcessor.cpp:623` |
| `TS_INFO` any other query | `FAIL` | `CommandProcessor.cpp:628` |
| `POS`, `IS_SPEAKING` | `speaking_code`: talking, receiving (`00`–`11`) | `CommandProcessor.cpp:89-101` |
| `POS`, `IS_SPEAKING` of an unknown client | `00` | `CommandProcessor.cpp:92-95` |
| `IS_SPEAKING_BULK` | one code and a tab per nickname, the last tab kept | `CommandProcessor.cpp:103-144` |
| a rejected message | `Error::sync_reply`: `00` for `POS` and `IS_SPEAKING`, `FAIL` for `TS_INFO`, the async text for the rest | |

```text
text ── parse ──► split at TAB (every field kept) ──► CommandName::from_wire_name
                                                          │ none ──► GameCommand::Unknown
                                                          ▼
                            MessageFields { command, tokens } ──► commands/<payload> parser
                                                          │
                                   GameCommand::<variant>(payload)  or  Error
```

`fields::MessageFields` wraps the tokens with the command they belong to, so every count check
and field read names the command and field in its error. The plugin answers a sync call with
`GameCommand::fixed_sync_reply` where no state is needed, from its state otherwise (the
`replies` builders), and with `Error::sync_reply` for a rejected message.

### Modules

#### `src/commands/`

Each parser takes the `MessageFields` of one message, checks its token count and reads the fields
at the original's indices (`tokens[i]` of `reference/tfar_1_0/ts/src/CommandProcessor.cpp`), so a
field's index in the code is its index in the C++. Lists inside a field (frequency entries,
speaker records, tower records, custom variables) are split with their own separators; a record
of the wrong shape is skipped where the original skips it (and, for speakers, where the original
abandons the rest of the list), and a malformed number rejects the message.

#### `src/fields/`

Every command parser reads its message through `MessageFields`: `require_count`,
`require_at_least` and `require_between` check the token count, and `text`, `nickname`, `truth`,
`number`, `integer` and `read` return a field or an error that names the command and the field.
The coordinate and vehicle readers return `None` for a malformed text; `MessageFields::read` turns
that into `Error::Malformed`. `NetworkObjectId::parse` never fails, as the original's `NetID`
never does: a text without a colon is the null id.

### Configuration

None: the crate reads no setting. `SETCFG` is parsed here and applied by
`tfar_plugin_logic_1_0::plugin_settings`.

### Public surface

- `parse`, `GameCommand`, `CommandName`.
- The payloads: `TsInfoQuery`, `PositionUpdate`, `FrequencyReport` (with `NICKNAMES_NEVER_ADOPTED`),
  `SpeakerRecord`, `TangentPress` and `TangentKind` (with `TANGENT_PRESSED`, `TANGENT_RELEASED`),
  `ConfigAssignment`, `TowerRecord`, `TrackingEvent`, `TrackerCustomVariable`,
  `VehicleDescriptor`, `NetworkObjectId`.
- `replies`: the reply strings, `speaking_code`, `speaking_code_text`, `bulk_speaking_reply`.
- `Error` (`WrongTokenCount`, `Malformed`, with `command` and `sync_reply`), `Result`,
  `prelude`.

### Boundaries

- Depends on: `tfar_foundation::arma_values` (numbers, frequency lists, nicknames, subtypes, separators);
  `tfar_foundation::geometry` (positions and directions); `tfar_plugin_logic_1_0::plugin_settings` (the `SETCFG` type tag);
  `thiserror` for the error type.
- Used by: `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_1_0`.
- Rules: a wrong token count or a malformed field rejects the message
  (`a_short_pos_message_is_an_error_not_an_out_of_bounds_read`); a short speaker record is skipped
  and the rest kept (`a_short_speaker_record_is_skipped_and_the_rest_kept`); the reply strings are
  the original's (`the_reply_strings_are_the_originals`).

## `plugin_commands`

The `tfar_plugin_logic_1_0::plugin_commands` crate: the 1.0 plugin-to-plugin commands sent through TeamSpeak,
byte-compatible with the original, and the TFAR block of a client's metadata. Used by
`tfar_plugin_1_0`.

### How it works

Plugins of the same mod talk through TeamSpeak's `sendPluginCommand`, TAB-separated
(`reference/tfar_1_0/ts/src/plugin.cpp:1026-1084`):

| Message | Tokens | Sent | Source |
|---|---|---|---|
| `TANGENT[_LR\|_DD]\tPRESSED\|RELEASED\tfreq\trange\tsubtype[\tclassname]\tsw\tdd\tnick` | 8, or 9 with the class name | to the channel on a press or release | `CommandProcessor.cpp:326` |
| `RELEASE_ALL_TANGENTS\tnick` | 2 | to the channel | `CommandProcessor.cpp:378` |
| `VOLUME\tnick\tvoiceVolume\ttrue\|false` | 4 | to the channel when the talking flag changes | `plugin.cpp:841` |
| `REQVOL\tclientId` | 2 | never; answered with `VOLUME` | `plugin.cpp:1056-1064` |

`decode` recognises a command by name and token count together, as the original does, and reads
a tangent's fields by position relative to its count: 8 tokens are sw 5, dd 6, nick 7; 9 tokens
are classname 5, sw 6, dd 7, nick 8 (the original reads the flags from 6 and 7 in both forms).
`encode` writes the original's bytes, the tangent's range as the game sent it, so decoding and
encoding gives the same text. `accepts_sender` keeps only commands from a plugin whose name starts
with `TFAR`.

The metadata block is published every 4 s between `<TFAR>` and `</TFAR>` in the client's
TeamSpeak metadata:

```text
Connected to Game: [B]Yes|No[/B]
Playing: [B]Yes|No[/B]
Plugin version: [B]<plugin version>[/B]
Addon version: [B]<addon version>[/B]
```

`format_metadata` gives that block, `merge_metadata` puts it into the existing metadata (or
removes it when empty), `metadata_says_playing` is the plugin-enabled check other clients run,
and `info_data_text` is the info panel's text, cut to the 512-byte buffer.

`command_codec.rs` matches the split tokens against each command's name and token count in one
slice pattern per command, so a count the original does not accept is `Error::Unrecognised`, and
reads the tangent's tail (`[sw, dd, nick]` or `[classname, sw, dd, nick]`) by its own layout.
`metadata.rs` works on whole metadata strings with `str::find` for the first tags, as the
original's `std::string::find`.

### Configuration

None: the crate reads no setting.

### Public surface

- `PluginCommand`, `TangentBroadcast`, `TangentKind`, `TeamspeakClientId`; `encode`, `decode`.
- `PLUGIN_NAME_PREFIX`, `accepts_sender`.
- `format_metadata`, `merge_metadata`, `metadata_block`, `metadata_says_playing`,
  `info_data_text`, `METADATA_START_TAG`, `METADATA_END_TAG`, `DISABLED_METADATA`,
  `PLAYING_METADATA_PREFIX`, `INFO_DATA_BUFFER_SIZE`.
- `Error` (`Unrecognised`, whose message is the original's `UNKNOWN PLUGIN COMMAND` log line, and
  `Malformed`), `Result`, `prelude`.

### Boundaries

- Depends on: `tfar_foundation::arma_values` (`RadioSubtype`, `parse_int`); `thiserror` for the error type.
- Used by: `tfar_plugin_1_0`.
- Rules: the encoded bytes match what the original 1.0 plugin sends
  (`every_command_round_trips_byte_for_byte`, `the_status_block_is_the_original_text`); an 8-token
  tangent reads its flags from tokens 5 and 6
  (`an_eight_token_tangent_reads_its_flags_from_tokens_five_and_six`).

## `client_state`

The `tfar_plugin_logic_1_0::client_state` crate: the per-client state of the 1.0 plugin: positions, radios,
tangents, vehicles and effect state, the local player's game data, the game link's connection
state, and the apply function of each game and plugin message. Used by `tfar_plugin_1_0`,
`tfar_plugin_logic_1_0::radio_routing`, `tfar_plugin_logic_1_0::voice_pipeline`.

### How it works

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

#### Lock and ownership contract

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

#### Fixed original bugs

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

`SessionState` (in `session/`) owns a `GameData` and a `ClientDirectory`. The directory maps a
nickname to its `ClientData` in a `BTreeMap` and a client id to its nickname in a `HashMap`, and
every change goes through it so the two always agree; the local player is the entry whose id the
directory marks as own. A `ClientData` keeps its position as reported and extrapolates it by the
vehicle velocity for the time since the last `POS` (`client_position(now)`); liveness uses a
separate activity clock, which tangents also refresh. Each `ClientData` owns a `ClientEffects`
(in `effects/`) and shares its vehicle descriptor (`Arc`). `reception.rs` holds `HeardVia`,
which the routing crate produces and the tangent handling and voice pipeline read; a `HeardVia`
shares its radio id and vehicle with the session, so making one copies no text. `GameLinkState` stands apart: one per plugin.

### Modules

#### `src/effects/`

`ClientEffects::ensure_dsp(requirements, context)` creates the two compressors (direct speech and
radio, so each envelope advances once per frame), reserves the object-interception buffer for
`frames_per_block`, and creates every required object that does not exist yet: radio effects by
radio id, HRTF sources by key (the HRIR sphere comes in `DspContext`), the can't-speak, vehicle
(per loss), speaker and phone filters by key, and the object-interception low-pass per object
count at the given strength. The audio path then borrows each object with a `*_mut(key)`
accessor, a `HashMap` lookup by `&str` that allocates nothing. `reset_radio_effects` and
`reset_voices` drop what the original's `resetRadioEffect` and `resetVoices` drop (the airborne
effects too).

The object-interception slots remember the strength they were designed with and are redesigned
when it changes; a cutoff `2000 − min(count, 5) · strength` at or below 0 Hz is
`ObjectInterceptionFilter::Silence`.

#### `src/session/`

Each apply function takes a parsed message (from `tfar_plugin_logic_1_0::game_commands` or
`tfar_plugin_logic_1_0::plugin_commands`) and `now`, writes it into the session, and returns what the app does
next in TeamSpeak. `apply_remote_tangent` takes the reception routing as a closure so this crate
stays below the routing crate: for a release the closure runs before the sender's state is reset,
with the sender's abilities as the ignore flags; for a press after. Only a real press or release
plays a remote sound, and the local player's received transmissions count each sender once.
`apply_local_tangent` takes the game's own tangent: only a change of the tangent flag acts, and it
returns the local sound and the broadcast for the app to play and send once the lock is released;
`apply_local_release_all_tangents` clears the flag, so the next press acts again.

### Configuration

- `pluginTimeout` (default 4 s, `tfar_plugin_logic_1_0::plugin_settings`): read by `GameLinkState::poll` and
  `is_connected`.
- `objectInterceptionStrength` (default 400): the caller passes it in `DspContext`.

### Public surface

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

### Boundaries

- Depends on: `tfar_foundation::geometry` (positions and directions); `tfar_foundation::arma_values` (frequency entries,
  stereo modes, subtypes); `tfar_plugin_logic_1_0::game_commands` (the parsed game messages, the speaking code);
  `tfar_plugin_logic_1_0::plugin_commands` (tangent broadcasts, client ids, the metadata test);
  `tfar_plugin_logic_1_0::radio_effects` (the radio effects); `tfar_plugin_logic_1_0::plugin_settings` (`pluginTimeout`);
  `tfar_audio::dsp_filters` (the filters); `tfar_audio::dynamics` (the compressors); `tfar_audio::hrtf` (the HRTF
  sources); `tfar_audio::sample_buffer` (the object-interception buffer); `thiserror`.
- Used by: `tfar_plugin_1_0`, `tfar_plugin_logic_1_0::radio_routing`, `tfar_plugin_logic_1_0::voice_pipeline`.
- Rules: effect state is only reached while its session's lock is held; clients are keyed by the
  full nickname (`nicknames_are_keyed_by_their_full_text_so_no_two_share_an_entry`); queries are
  pure (`is_alive_is_a_pure_query_and_never_changes_the_data_frame`).

## `radio_routing`

The `tfar_plugin_logic_1_0::radio_routing` crate: whether and how a 1.0 listener hears a speaker over radio,
intercom or speaker radio, the radio effect error levels, the serious-mode mute rules and the
serious channel lookup. Used by `tfar_plugin_1_0`, `tfar_plugin_logic_1_0::voice_pipeline`.

### How it works

Every rule reads a `RoutingContext` (the session, the settings, the antenna manager and `now`)
built under the session lock, and never changes state.

`is_over_radio(context, sender, ignore)` lists every `HeardVia` of the sender for the local player,
in the order of `isOverRadio` (`reference/tfar_1_0/ts/src/clientData.cpp:267-364`):

1. **Intercom** (checked first): intercom enabled, the sender holds no tangent, both in the same
   vehicle (not `no`) on the same intercom slot (not -1).
2. A sender with range 0 (no tangent) is heard on nothing more.
3. **Carried radio** (`is_over_local_radio`, `clientData.cpp:148-265`), unless the sender is the
   listener. The short-range tangent (or flag) with a short-range or diver radio sends as short
   range, or as a diver radio when the sender is under water without a short-range radio; else the
   long-range tangent (or flag) with a long-range radio. The frequency must be on one of the
   listener's radios. Under water the submerged part of the path is stretched to the diver range
   (`70 + 230 · (1 − waves)`); beyond range a tower found by `find_connection` may still carry it
   if the sender reaches the tower. The long-range radio is preferred when the listener can use
   it; in half-duplex mode the radio being transmitted on (by class name) hears nothing.
4. Beyond its range, with no tower, the sender is heard on nothing more.
5. **Speakers** on the exact frequency, unless the speaker is the sender's own, its position is
   unknown (on an unknown owner), or it is under water (height below 0).

`is_over_radio_into(context, sender, ignore, out)` does the same into a list the caller reuses,
which it clears first; every `HeardVia` shares its radio id and vehicle with the session (`Arc`),
so a list with room for every reception routes a frame without allocating. `is_over_radio` wraps
it.

`mute_decision` follows `setGameClientMuteStatus` (`plugin.cpp:68-94`); `MuteTracker` records who
this plugin muted, so requests go out only on a change and `unmute_all` returns the tracked set;
`killed_mute_matrix` follows `processUnitKilled` (`CommandProcessor.cpp:552-587`).
`serious_mode_channel` resolves `ID:<n>` or an exact channel name. `effect_error_from_distance`,
`capped_by_antenna` and `underwater_effect_error` give the voice pipeline the error level of a
radio effect, with no antenna read as the original's loss of 7. `apply_remote_tangent` runs
`SessionState::apply_remote_tangent` with `is_over_radio` as its routing.

`reception/` turns a `RoutingContext` and a sender into `HeardVia` values; `mute/` decides mutes
on top of it; `effect_error.rs` and `underwater.rs` hold the formulas the voice pipeline shares
with the rules. `remote_tangent.rs` wires `is_over_radio` into the client state's tangent
handling, which cannot depend on this crate.

### Modules

#### `src/mute/`

The plugin app asks `mute_decision` after a client's position, volume or tangent and
`killed_mute_matrix` after a kill, feeds the answers to its `MuteTracker`, and sends TeamSpeak only
the changes the tracker reports. On game end or disconnect `unmute_all` returns every client the
plugin muted; a client that leaves is forgotten with `client_left`.

#### `src/reception/`

`is_over_radio` checks the intercom, stops for a sender without range, finds the best tower with
`AntennaManager::find_connection` (not for the listener itself), adds the carried-radio reception,
stops when the sender is beyond range without a tower, and adds the speakers with the tower's loss.
Each `HeardVia` carries the tower's loss when one connects the two, and shares its radio id and
vehicle with the session; `is_over_radio_into` fills a reused list, so routing a frame allocates
nothing once the list holds every reception.

### Configuration

Read from `tfar_plugin_logic_1_0::plugin_settings::PluginSettings` through the context:

- `intercomEnabled` (default true): the intercom rule.
- `full_duplex` (default true): off, the radio being transmitted on hears nothing.
- `disableAutomaticMute` (default false): on, `mute_decision` and `killed_mute_matrix` leave
  serious-mode mutes alone.

### Public surface

- `RoutingContext`; `is_over_radio`, `is_over_radio_into`, `is_over_local_radio`;
  `apply_remote_tangent`.
- `mute_decision`, `MuteDecision`, `VOICE_RANGE_MARGIN`; `MuteTracker`; `killed_mute_matrix`,
  `MuteChanges`.
- `serious_mode_channel`, `ChannelId`, `CHANNEL_ID_PREFIX`.
- `effect_error_from_distance`, `capped_by_antenna`, `underwater_effect_error`;
  `diver_radio_distance`, `underwater_weighted_distance`; `prelude` adds `distance_underwater`.
- `Error`, `Result`.

### Boundaries

- Depends on: `tfar_plugin_logic_1_0::client_state` (the session, the clients and `HeardVia`);
  `tfar_plugin_logic_1_0::antennas` (the towers); `tfar_plugin_logic_1_0::plugin_settings` (the settings above);
  `tfar_foundation::geometry` (distances, `distance_underwater`); `tfar_foundation::arma_values` (the `ID:<n>` integer);
  `thiserror`.
- Used by: `tfar_plugin_1_0`, `tfar_plugin_logic_1_0::voice_pipeline`.
- Rules: routing reads client state and never changes it; the listener never hears itself on its
  own radio (`the_listener_never_hears_itself_on_its_own_radio`) and is never muted
  (`the_local_player_is_never_muted`).

## `radio_effects`

The `tfar_plugin_logic_1_0::radio_effects` crate: the 1.0 radio sound chains: filters, compressor input and noise
per radio type. Used by `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_logic_1_0::voice_pipeline`.

### How it works

The crate ports `reference/tfar_1_0/ts/src/RadioEffect.hpp`, its `processRadioEffect` mixdown
included. A voice pipeline owns one effect per sender and radio: [SW](/documentation/glossary.md)
radios use `PersonalRadioEffect` (or `UnderWaterRadioEffect` when carried under water), airborne
radios `AirborneRadioEffect`, and [LR](/documentation/glossary.md) radios and the intercom
`LongRangeRadioEffect`. It sets the error level from the distance, then hands
`process_radio_effect` the block copied for that radio:

```text
stereo i16 block --(left + right) x gain / (2 x 32766)--> mono f32 block --effect--> clamp, x32766 --> one or both ears
```

The three speaker effects run, per sample, foldback distortion, ×30, a 2399-sample delay and ring
modulation by the error level, then an [RBJ](/documentation/glossary.md) high-pass and low-pass. The
underwater effect drops samples at random with about the error level as probability, band-passes
round 1000 Hz and multiplies by 30. Effects are built off the TeamSpeak audio thread; processing
runs on it and allocates nothing.

`process_radio_effect` multiplies the gain by 1.5 for one ear, mixes each stereo frame to
`(left + right) · (1/2 · gain · 1/32766)` in `f32` in the `MonoMixBuffer`, calls
`RadioEffect::process` on the whole block, and writes `trunc(clamp(sample, −1, 1) · 32766)` back
through `tfar_audio::sample_buffer::float_to_sample` to one or both channels. The effects in `effects/` are
built from `signal_stages/` and the `MonoFilter` of `tfar_audio::dsp_filters`; the speaker effects map
their error level through `error_level.rs`, the underwater effect uses the raw value. Each filter
output is rounded back to `f32`, as the original's float buffers are.
`tests/signal_measurements.rs` holds the sines, RMS, tone share and band-shape helpers the effect
tests share.

### Modules

#### `src/effects/`

`speaker_chain.rs` computes the foldback threshold `0.3 · (1 − error) · mean(|x|) / 0.005` from the
block as it arrives, then runs each sample through the foldback, ×30, the delay line and the ring
modulator, and the result through the RBJ high-pass and low-pass. The three speaker effects differ
only in their filters; the personal effect has no microphone filters. `under_water.rs` draws one
dropout decision per sample, then band-passes and multiplies by 30.

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
  `AirborneRadioEffect`, `PersonalRadioEffect` and `UnderWaterRadioEffect`; `SAMPLE_RATE_HZ`.
- `error_level_from_distance`: the 1.0 error-level curve.
- `process_radio_effect` and its reusable `MonoMixBuffer`.
- `Error`, `Result` and `prelude` (which also re-exports `StereoMode`).

### Boundaries

- Depends on: `tfar_audio::dsp_filters` (the Butterworth and RBJ filters); `tfar_audio::sample_buffer` (the
  stereo block and the `f32` to `i16` conversion); `tfar_foundation::arma_values` (`StereoMode`); `fastrand`
  for the seeded dropouts; `thiserror` for the error type.
- Used by: `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_logic_1_0::voice_pipeline`.
- Rules: the error level rises with distance, never falls, and stays inside its table
  (`src/tests/error_level.rs`); each effect keeps the original's processing order and filters
  (the tests in `src/effects/tests/`).

## `voice_pipeline`

The `tfar_plugin_logic_1_0::voice_pipeline` crate: the 1.0 audio path: one TeamSpeak voice frame through routing,
effects, compressor, HRTF and mixing. Used by `tfar_plugin_1_0`.

### How it works

The crate is `processVoiceData` of `reference/tfar_1_0/ts/src/plugin.cpp:455-743` for a frame the
plugin app has already copied to stereo, with the session locked. Step by step:

1. **Gates.** Outside a game, for an unknown sender, or for a sender (or listener) without TFAR the
   frame passes through untouched; the local player's own voice without TFAR is silence;
   `muteSpectators` silences one spectator for another; a sender the spectator rules keep away is
   silence in serious mode when the listener is dead or the sender is not alive.
2. **Voice multiplier**, saturating; the result is kept as the radio original.
3. **Direct speech** (not the local player, not a spectator, within voice range + 15 m, 2 m per
   object): clear, the object low-pass with the previous frame as look-back, the vehicle low-pass,
   or the 100 Hz can't-speak low-pass, each at the original's gain; then the placement; then the
   direct-speech compressor. A sender not heard directly is silenced unless both spectate.
4. **Radios.** The radio compressor (its own instance) runs once over the original; each
   reception of `is_over_radio_into` (refilled into the scratch reception list, each reception
   sharing its radio id and vehicle with the session) gets a copy through its path (carried radio
   by subtype, speaker radio, intercom) and is mixed in, saturating. The reception and route plan
   lists are emptied at the end of the frame, under the lock, so the audio thread never drops the
   last handle on a shared id.
5. **Global volume.**

`process_own_voice_for_speakers` runs the same chain on the captured voice as the local player's,
microphone path (no direct speech, no intercom), and returns the block for the `my_radio_voice`
stream; `append_own_voice_for_speakers` appends it to a `PlaybackMixer`. `ensure_sender_dsp` plans
a sender exactly as the audio path does and creates the objects its frames will borrow, off the
audio thread.

`render_remote_tangent_sound` shapes a decoded remote tangent WAV for one
`RemoteSoundPlacement` of `SessionState::apply_remote_tangent`, as the positional `playWavFile`
does (`reference/tfar_1_0/ts/src/PlaybackHandler.cpp:147-262`): the gain, then, with the local
player known, the vehicle low-pass, and on a loudspeaker the distance attenuation, the speaker
band-pass, the under-water low-pass at `CANT_SPEAK_GAIN * 50` and the simple ILD; a carried radio
keeps only its ear (`apply_stereo_mode`). The radio sounds are plugin-folder files, which the
original places by the ILD alone, so it takes no HRIR sphere. It builds its filters per sound and
runs off the audio thread.

#### Level and direction (replacing X3DAudio and clunk)

- **Direct speech:** the level is the X3DAudio part kept as gains: the 25-point voice curve
  (`DistanceCurve::tfar_voice` over `volume_attenuation`) at the straight distance, scaled by the
  sender's voice range, times `EmitterCone::TFAR_VOICE` when `voiceCone` is on. The original's
  swapped listener and emitter and its negated facing x and y (`helpers.cpp:46-110`) are kept, so
  the cone measures the sender's facing towards the listener. The direction is the HRTF source
  `voice_clunk` of the sender (`tfar_audio::hrtf`), which replaces both clunk's ear delay and X3DAudio's
  panning; the simple ILD is not used for direct speech.
- **Makeup gain:** `HRTF_MAKEUP_GAIN_DB` (+4 dB) follows the HRTF on direct speech, compensating
  the diffuse-field EQ of the IRCAM sphere (about −4 dB straight ahead), so speech straight ahead
  keeps its original loudness. Whether to keep it is the roadmap's open question for the owner.
- **Speaker radios:** the original ran clunk and the simple ILD; here the HRTF source of the
  speaker's radio id gives direction and the ILD is dropped. They get no makeup: the ILD they
  replace is 0.625 (−4.1 dB) straight ahead, about what the HRTF leaves.
- **No HRTF source** (sphere missing, or not yet ensured): the 1.0 simple ILD
  (`ILD_COEFFICIENT_1_0`) pans, at the same level without the makeup.
- **Distance gain:** `volume_attenuation` is `10^(−1.5 · d / max)` (the original's
  `(d / (2 · max)) · −60 / 20`), 1 within 1 m, 0 below 0.001, `max` = 5 m for a player who cannot
  speak.

#### Where DSP objects are missing

The audio path never creates anything: `ClientEffects`' accessors answer `None` for an object
never ensured. A missing low-pass leaves its gain, a missing HRTF source falls back to the ILD, a
missing compressor leaves the frame uncompressed, and a reception whose radio effect, phone
band-pass or speaker band-pass is missing stays out of the mix (a radio without its effect would
sound like direct speech from nowhere). The voice HRTF source (about 2 MB) is ensured only while
the sender is within direct-speech range.

#### Fixed original bugs

| Original bug | Test |
|---|---|
| direct speech and radio share one compressor | `direct_speech_and_radio_each_advance_their_own_compressor_once_per_frame` |
| the first frame behind objects reads an empty look-back buffer | `the_first_frame_behind_objects_has_no_look_back_and_is_filtered_once` |
| a speaker's muffling skipped when the sender shares the listener's vehicle | `a_speaker_outside_the_listeners_vehicle_is_muffled_even_when_the_sender_sits_inside` |
| the intercom never ducks | `the_intercom_is_ducked_while_a_long_range_transmission_is_received` |

`VoiceProcessor::process_sender` runs in two phases. With the session borrowed immutably,
`sender_gate` decides pass-through, silence or processing, and `FramePlan::build` reads a `Scene`,
plans direct speech (`direct_speech/`), lists the receptions with `is_over_radio_into` and plans
each (`radio_paths/`). Then, with the sender's `ClientEffects` borrowed mutably, the frame gets the voice
multiplier, is copied as the radio original, goes through direct speech (or is silenced when the
sender is not heard directly, unless both spectate), and every reception's copy of the
radio-compressed original goes through its path and is mixed in; the global volume comes last. All
blocks live in `VoiceScratch`. `dsp_needs` runs the same planning off the audio thread and hands
`ClientEffects::ensure_dsp` the objects the next frames will look up. `remote_tangent_sound.rs`
shapes a remote tangent WAV into a playback on the command thread.

### Modules

#### `src/direct_speech/`

`plan_direct_speech` reads a `Scene` and decides, as `reference/tfar_1_0/ts/src/plugin.cpp:547-597`
does, whether the sender is heard directly (not the local player, not a spectator, not kept away by
the spectator rules, within the voice range plus 15 m with 2 m per object) and how: clear, behind
objects (the object low-pass with a one-frame look-back), from another vehicle (the vehicle low-pass
at `volumeAttenuation(d, …, 1 − loss) · (1 − loss)^1.2`) or muffled because one of the two cannot
speak (the 100 Hz low-pass at `volumeAttenuation(d, false, …) · 14`). The placement's level is the
X3DAudio replacement: the 25-point voice curve of `tfar_audio::spatial_audio` at the straight distance,
scaled by the sender's voice range, times the voice cone when `voiceCone` is on, with the original's
swapped roles and negated facing so a sender facing the listener gets the inner volume.
`render_direct_speech` runs that plan over the frame with the sender's DSP objects and ends with the
direct-speech compressor.

#### `src/radio_paths/`

`plan_routes` turns each `HeardVia` of `is_over_radio` into a `RoutePlan`, as the loop of
`reference/tfar_1_0/ts/src/plugin.cpp:600-737` decides it. A carried radio plays at
`((volume + 1) / 10)^4` (a tenth with `headsetLowered`) through the effect of the sender's subtype:
`digital` the personal effect (the underwater effect when heard as a diver radio), `airborne` and
`digital_lr` their effects, `phone` the phone band-pass at ×10, anything else nothing. The error is
the routing crate's distance error capped by the antenna loss (`None` read as 7). A speaker radio
gets the band-pass at ×4, its distance gain over `volume / 10 · speakerDistance` (or the vehicle
low-pass when a vehicle stands between it and the listener), the 100 Hz low-pass at ×14 under
water, the personal effect and its placement. The intercom plays the long-range effect at error 0
and the `intercomVolume`, ducked by `intercomDucking` while a long-range transmission above volume
2 is received. `render_route` runs a plan over the compressed copy of the frame and says whether to
mix it in.

### Configuration

Read from `tfar_plugin_logic_1_0::plugin_settings::PluginSettings` through `VoiceFrameContext`:

- `voiceCone` (default true): the emitter cone on direct speech.
- `headsetLowered` (default false): carried radios and the intercom at a tenth.
- `intercomVolume` (default 0.1), `intercomDucking` (default 0.2): the intercom level and ducking.
- `muteSpectators`, `spectatorNotHearEnemies`, `spectatorCanHearFriendlies`: the spectator gates.
- `objectInterceptionStrength` (default 400): read by the caller into `DspContext`.
- Through the routing: `intercomEnabled`, `full_duplex`.

### Public surface

- `VoiceProcessor`: `new(max_frames)`, `process_sender`, `process_own_voice_for_speakers`,
  `append_own_voice_for_speakers`, `scratch_capacities`; `VoiceScratchCapacities`;
  `OWN_RADIO_VOICE_STREAM`.
- `VoiceFrameContext` (settings, antennas, in game, serious mode, sender metadata, now) and
  `VoiceOutcome` (`Processed`, `Silenced`, `PassThrough`).
- `ensure_sender_dsp`, `ensure_every_sender_dsp`.
- `render_remote_tangent_sound`, `RemoteSoundPlan`, `RemoteSoundListener`,
  `UNDER_WATER_SPEAKER_GAIN`, `apply_stereo_mode`; used by `tfar_plugin_1_0` for the remote and
  local tangent sounds.
- `volume_attenuation`, `vehicle_volume_loss`, `vehicle_loss_gain`, `speaker_audible_distance`,
  `CANT_SPEAK_DISTANCE`, `CANT_SPEAK_GAIN`, `SPEAKER_GAIN`, `UNDERWATER_LEVEL`, `MAX_VEHICLE_LOSS`,
  `NO_VEHICLE_LOSS`; `HRTF_MAKEUP_GAIN_DB`, `hrtf_makeup_gain`.
- `Error` (`Dsp`, `Mixer`, `RemoteTangentSound`), `Result` and `prelude`.

### Boundaries

- Depends on: `tfar_plugin_logic_1_0::client_state` (the session, the effects and their keys);
  `tfar_plugin_logic_1_0::radio_routing` (`is_over_radio_into`, the error levels); `tfar_plugin_logic_1_0::radio_effects` (the
  effect mixdown); `tfar_plugin_logic_1_0::plugin_settings` (the settings); `tfar_plugin_logic_1_0::antennas` (the towers the
  routing reads); `tfar_foundation::geometry` (positions and directions); `tfar_audio::dsp_filters` (the filters);
  `tfar_audio::dynamics` (the compressors); `tfar_audio::hrtf` (the HRTF sources and directions);
  `tfar_audio::spatial_audio` (curve, cone, simple ILD); `tfar_audio::sample_buffer` (the frame, gain and mixing);
  `tfar_audio::playback_mixer` (the `my_radio_voice` stream and the remote sound's playback); `thiserror`
  for the error type; `tracing` for a refused remote sound filter.
- Used by: `tfar_plugin_1_0`.
- Rules: each compressor runs once per frame
  (`direct_speech_and_radio_each_advance_their_own_compressor_once_per_frame`); the scratch never
  grows and no reception outlives its frame
  (`a_thousand_frames_leave_every_scratch_capacity_unchanged`); nothing pans twice.
