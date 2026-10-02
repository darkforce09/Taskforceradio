**Status:** live

# M7 · W1 · `tfar_plugin_settings_1_0`, `tfar_game_commands_1_0`, `tfar_plugin_commands_1_0`

You are agent W1, the 1.0 command-language author for the TFAR Rust port. Read
`/documentation/program/agent_briefs/shared_brief.md` first (including the **Build directory rule**), then
`/documentation/standards/coding_standards.md` and **`/documentation/program/behaviour_specs/tfar_1_0_plugin.md` §2, §4, §5 and §8**.
That spec is the source of truth. Also read `/documentation/program/bug_fixes.md` (F-003, F-014 and the others that name your
crates) and the `tfar_arma_values` / `tfar_geometry` APIs.

## Goal
Pure parsing, formatting and settings, with no TeamSpeak, no threads and no global state. The plugin app (W6) does the
side effects.

## `tfar_plugin_settings_1_0`
- The 20 settings of spec §5, with their defaults, in a `PluginSettings` struct.
- `set(key, value_text, type_tag) -> Result<SettingChanged>` follows SETCFG semantics:
  - **Key matching:** case-insensitive *prefix* match over the setting name's length (`_strnicmp`, `reference/tfar_1_0/ts/src/settings.hpp:47`). Keep the original rule so the addon's keys still match, and document it.
  - **Type tags:** `BOOL` is true for `true`/`TRUE`; `SCALAR` is parsed as a number; anything else is a string. The value's type follows the setting's default, and cross-type conversion is bool ← number > 0.
  - **Errors:** an unknown key is `Error::UnknownKey` (the plugin prints the original message "Used invalid config key: …"). A wrong token count gives the original's empty-string fallback, explicitly.
- `needs_refresh` flag semantics, from the spec's NEEDCFG section: it starts true and becomes false on any set. The plugin sets it true again on game disconnect, via `mark_needs_refresh()`.
- Typed getters per setting, e.g. `serious_channel_name()`, `plugin_timeout() -> Duration`, `voice_cone()`.

## `tfar_game_commands_1_0`
- **Parsing:** `parse(text) -> Result<GameCommand, CommandError>`, splitting with `split_fields`.
- **Variants** (spec §2.2), with the hash-switch semantics replaced by exact name matching:
  - `TsInfo(Server | ServerUid | Channel | ChannelId | Ping | Version | Other)`
  - `Position(…14 fields…)` — F-003: count checked; vehicle id parsed into `VehicleDescriptor { name, isolation, intercom_slot, velocity }` per spec §5 (`name\x10isolation|"turnout"\x10slot[\x10velocity]` or `"no"`).
  - `IsSpeaking`, `IsSpeakingBulk(Vec<nick>)`
  - `Frequencies(…)` — `ThreeOrFourParts` entries.
  - `Killed`, `DataFrame`
  - `Speakers` — F-?: a record with fewer than 6 parts is skipped instead of aborting the rest (spec §2.2, OF-008; add this fix to your report as a proposed new F-id).
  - `Tangent { kind, pressed, frequency, range, subtype, radio_classname: Option }`
  - `ReleaseAllTangents`, `SetConfig { key, value, type_tag }`, `MissionEnd`
  - `RadioTowerAdd(Vec<TowerRecord { net_id, range, position }>)`, `RadioTowerRemove(Vec<net_id>)`
  - `Track { action, uid, custom_vars }`, `CollectDebugInfo`
  - `Unknown`
- **Sync vs async:** `GameCommand::is_async_only()` encodes the original sync/async rules. Replies:
  - `UNKNOWN COMMAND`, `ASYNC COMMAND SENT IN SYNC CONTEXT`, `FAIL`, `PONG`, `ERROR_GETTING_SERVER_NAME`.
  - `speaking_code(talking: bool, receiving: bool) -> [u8; 2]` for the 2-character code; `"00"` for an unknown client.
  - the bulk format (`"XX\t"` per nickname, trailing tab kept).

## `tfar_plugin_commands_1_0`
- **The 1.0 client-to-client messages** (spec §4 table): TANGENT (8 or 9 tokens; F-014 fixes the 8-token index shift, so 8 tokens means no classname), RELEASE_ALL_TANGENTS, VOLUME, REQVOL.
- **API:** `encode`/`decode`, byte-compatible with Windows 1.0 players. Accept filter `PLUGIN_NAME_PREFIX = "TFAR"`, prefix match.
- **Metadata:**
  - `format_metadata(connected, playing, plugin_version, addon_version)` gives the exact `<TFAR>Connected to Game: [B]Yes|No[/B]\nPlaying: …\n</TFAR>` text.
  - `merge_metadata`, `metadata_says_playing` (prefix `"Connected to Game: [B]Yes[/B]\nPlaying: [B]Yes[/B]\n"`), `DISABLED_METADATA`, `info_data_text`.
- **Tests:** round trips; byte-exact strings with cited C++ lines; the 8- and 9-token TANGENT; the sender filter; metadata merge.

## Owned files
`crates/version_1_0/tfar_plugin_settings_1_0/**`, `crates/version_1_0/tfar_game_commands_1_0/**`, `crates/version_1_0/tfar_plugin_commands_1_0/**`.
Root manifest: one Edit at most (re-read first), reported.

## Checks (each alone, with your target dir; logs `target/agent-logs/w1-*.log`)
fmt, clippy `--all-targets -D warnings`, test for the 3 crates; `cargo xtask verify all` (report foreign violations only).

## Report additions
Public API list; settings table with defaults; reply table; F-ids pinned, by test name.

Budget: L (350k tokens). Stop there and report what is done and what is not.
