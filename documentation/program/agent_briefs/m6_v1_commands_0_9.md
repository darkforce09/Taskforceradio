**Status:** live

# M6 · V1 · `tfar_game_commands_0_9` and `tfar_plugin_commands_0_9`

You are agent V1, the 0.9 command-language author for the TFAR Rust port. Read
`/documentation/program/agent_briefs/shared_brief.md` first (including the **Build directory rule**), then
`/documentation/standards/coding_standards.md` and **`/documentation/program/behaviour_specs/tfar_0_9_plugin.md` §2 and §4**.
That spec is the source of truth for the wire formats. Also read `/documentation/program/bug_fixes.md` (F-003, F-020, F-021)
and the `tfar_arma_values` / `tfar_geometry` APIs.

## Goal
Pure, allocation-light parsing of every 0.9 game message into typed values, plus reply formatting, and encoding and
decoding of every 0.9 TeamSpeak plugin command. **No TeamSpeak, no threads, no state.** The plugin app (V5) does the
side effects.

## `tfar_game_commands_0_9`
- **Parsing:** `parse(text: &str) -> Result<GameCommand, CommandError>`, splitting with `tfar_arma_values::split_fields` on TAB (getline semantics).
- **Variants**, with the exact token-count rules of the spec table:
  - `TsInfo(TsInfoQuery { Server, Channel, Ping, Other(String) })`
  - `Killed { nickname }` — F-020: the spec's `>2` bug fixed to accept the 2 tokens Arma sends.
  - `Speakers(Vec<SpeakerRecord>)` — VT/LF records, empty records skipped, freqs split on `|`, optional position via `parse_optional_position`, waveZ default 1.
  - `ReleaseAllTangents { nickname }`
  - `Version { addon_version, serious_channel_name, serious_channel_password }`
  - `Track { path }`
  - `Position(PositionUpdate { … 13 fields })`
  - `Tangent { kind: TangentKind { ShortRange, LongRange, DiverDigital }, pressed, frequency, range, subtype: RadioSubtype }`
  - `Frequencies(FrequencyUpdate { sw: Vec<FrequencyEntry>, lr, dd_frequency, alive, my_voice_volume, dd_volume_level, nickname, waves_level, terrain_coefficient, global_volume, receiving_distance_multiplier, speaker_distance })` — field `[11]` is ignored, as the original does; document it.
  - `IsSpeaking { nickname }`
  - `Unknown(String)`
- **Errors (F-003):** every number goes through `tfar_arma_values`, and a malformed field gives `CommandError::Malformed { command, field }`, never a crash.
- **Reply strings** as constants and builders: `OK`, `FAIL`, `DONE`, `PONG`, `SPEAKING`, `NOT_SPEAKING`, `ERROR_GETTING_SERVER_NAME`.
  - `fn reply_for_unhandled(cmd) -> &'static str` captures the original's per-command reply. F-021: `RELEASE_ALL_TANGENTS` and `TRACK` reply `OK` instead of falling through to `FAIL`. Document every reply in a table in the crate README.
- **Wire constant:** `IN_GAME_TRIGGER`. Any command except VERSION counts as "in game" (F-022 is applied in client_state; expose `GameCommand::counts_as_game_activity()`).

## `tfar_plugin_commands_0_9`
- **The 0.9 client-to-client messages** (spec §4 "Plugin commands"):
  - `TANGENT`/`TANGENT_LR`/`TANGENT_DD` (6 tokens: type, PRESSED/RELEASED, freq, range, subtype, nickname)
  - `RELEASE_ALL_TANGENTS` (2)
  - `VOLUME` (4: nick, voiceVolume, true/false)
- **API:** `encode(&PluginCommand) -> String` and `decode(&str) -> Result<PluginCommand>`, byte-compatible with Windows 0.9.12 players.
- **Accept filter:** `PLUGIN_NAME_PREFIX = "task_force_radio"` and `accepts_sender(plugin_name) -> bool` (`strncmp(…, 16)` semantics, `reference/tfar_0_9_12/ts/src/plugin.cpp:3112`).
- **Metadata block** (spec §4 "Metadata"):
  - `format_metadata(connected: bool, playing: bool, plugin_version, addon_version) -> String` reproduces the exact `<TFAR>…</TFAR>` text, including the doubled `[/B]` (wire compatibility with Windows players' `isPluginEnabledForUser`).
  - `merge_metadata(existing, block) -> String` replaces or inserts the block and keeps the surrounding text.
  - `metadata_says_playing(client_metadata) -> bool` checks the `[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]Y[/B]` prefix.
  - `DISABLED_METADATA`.
  - `info_data_text(metadata) -> Option<&str>` for `ts3plugin_infoData`.
- **Tests:** a round trip for every message; byte-exact strings against hand-written expected values taken from the C++ code (cite lines); the sender filter; metadata merge with surrounding text and with a missing block.

## Owned files
`crates/version_0_9/tfar_game_commands_0_9/**`, `crates/version_0_9/tfar_plugin_commands_0_9/**`. Root manifest: one Edit at most
(re-read first), reported.

## Checks (each alone, with your target dir; logs `target/agent-logs/v1-*.log`)
fmt, clippy `--all-targets -D warnings`, test; `cargo xtask verify all` (report foreign violations only).

## Report additions
Public API list; the reply table; the F-ids pinned, by test name.

Budget: M (250k tokens). Stop there and report what is done and what is not.
