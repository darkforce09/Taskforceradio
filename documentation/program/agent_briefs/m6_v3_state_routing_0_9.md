**Status:** live

# M6 · V3 · `tfar_client_state_0_9` and `tfar_radio_routing_0_9`

You are agent V3, the 0.9 state-and-routing author for the TFAR Rust port. You own both crates. Read
`/documentation/program/agent_briefs/shared_brief.md` first (including the **Build directory rule**), then
`/documentation/standards/coding_standards.md`, **`/documentation/program/behaviour_specs/tfar_0_9_plugin.md` §1–§4**,
`/documentation/program/bug_fixes.md` (F-002, F-022, F-023, F-024, F-025) and the original
`reference/tfar_0_9_12/ts/src/plugin.cpp`. Use the public APIs of `tfar_game_commands_0_9` (typed commands),
`tfar_radio_effects_0_9`, `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf` and `tfar_geometry`.

## Design rules
- These crates are **pure state and decisions**, with no TeamSpeak calls and no threads. The app (V5) owns the lock (one `parking_lot::Mutex<ServerRadioData>` per server connection, F-002) and turns the decisions into TS calls.
- Time is injected: `now: Instant` parameters, never `Instant::now()` inside, so tests are deterministic.
- DSP objects are owned by the client entry and created lazily **when the app asks off the audio thread**: `ensure_dsp(...)` during command handling. The audio path only borrows them, so the audio path allocates nothing.

## `tfar_client_state_0_9`
- **`ServerRadioData`** holds every field of the spec's `SERVER_RADIO_DATA` (P:370-441):
  - my nickname, original nickname (F-024: saved once per game session), tangent pressed, my position, view angle and can-speak;
  - SW/LR frequency maps, DD frequency;
  - `speakers` (frequency → records), levels, the serious channel name/password, the addon version;
  - `current_data_frame`.
- **`ClientData`**, per nickname (`HashMap<String, ClientData>`), holds every `CLIENT_DATA` field plus its DSP state:
  - SW/LR/DD effect maps by radio id;
  - the HRTF sources by key (voice, per radio id), which replace the clunks;
  - the cant-speak / vehicle / speaker / phone filters by key;
  - the compressor (`VERSION_0_9` preset).
  - `reset_radio_effects()` and `reset_voices()` as in the spec.
- **Applying commands:**
  - `apply_position`, `apply_frequencies` (returns an optional `RenameRequest`), `apply_version`, `apply_speakers`, `apply_killed`;
  - `apply_remote_tangent(...)`, `apply_remote_volume(...)`, `apply_release_all_tangents(...)`, for incoming plugin commands;
  - `has_fresh_data(nickname)`: the data-frame window ±1 plus a matching client id;
  - `remove_expired(now)`: 20 s, or a frame difference > 1.
  - The game-session flag: `GameSession { in_game, last_game_activity }`. F-022: in game on the first non-VERSION command of a session; `game_timed_out(now)` after 4 s without one.
- **Vehicle descriptor parse** (0.9 rules, P:967): `_turnout`, last `_`, or whole id, giving `{ name, isolation }`.
- **`update_nicknames(&[(ClientId, String)])`** for the channel client list.

## `tfar_radio_routing_0_9`
- **`effective_distance`** (P:954), **`diver_radio_distance`** (P:932), **`effect_error_from_distance`** (P:941).
- **`is_over_local_radio`** (P:989) and **`is_over_radio`** (P:1091): exact check order, overwrite semantics, and ignore flags. They return `Vec<HeardVia { over, on, volume, stereo_mode, radio_id, position, wave_z, vehicle }>`.
- **Mute decisions** (P:1188, P:1356). These are pure functions returning `MuteDecision { mute: Vec<ClientId>, unmute: Vec<ClientId> }` against a tracked `MuteTracker` (the set this plugin muted):
  - F-025: unmute only what we muted, and only on change;
  - F-023: a missing entry is muted without touching voice state;
  - `set_mute_for_dead_players`.
- **Serious-mode helpers:** `is_serious_mode(channel_name_of_client, configured_name)`; the other-radio-plugin channel list (P:896).

## Tests
- State: every command application.
- F-022: in game without a serious channel.
- F-024: the original nickname after two renames.
- Expiry, and the frame window.
- Routing: a table-driven case set per spec rule, covering SW→SW, SW on LR list, LR→SW, LR→LR, DD in and out of range, a speaker on frequency owned by another, a stale owner position, and the ignore flags.
- Mute decisions: F-023, F-025, dead players.
- Each F-id gets a named test.

## Owned files
`crates/version_0_9/tfar_client_state_0_9/**`, `crates/version_0_9/tfar_radio_routing_0_9/**`. Root manifest: one Edit at most (re-read first), reported.

## Checks (each alone, with your target dir; logs `target/agent-logs/v3-*.log`)
fmt, clippy `--all-targets -D warnings`, test for both crates; `cargo xtask verify all` (report foreign violations only).

## Report additions
Public API per crate; F-ids pinned, by test name; the lock and ownership contract the app must follow.

Budget: L (350k tokens). Stop there and report what is done and what is not.
