**Status:** live

# M7 · W3 · `tfar_antennas_1_0`, `tfar_client_state_1_0`, `tfar_radio_routing_1_0`

You are agent W3, the 1.0 state, antenna and routing author for the TFAR Rust port. You own the three crates. Read
`/documentation/program/agent_briefs/shared_brief.md` first (including the **Build directory rule**), then
`/documentation/standards/coding_standards.md`, **`/documentation/program/behaviour_specs/tfar_1_0_plugin.md` §1, §3.8, §5
and §8**, `/documentation/program/bug_fixes.md` (F-002, F-008, F-015, F-016, F-017), and the originals
`reference/tfar_1_0/ts/src/{clientData,serverData,antennaManager,task_force_radio,datatypes}.{cpp,hpp,h}`. Use the public
APIs of `tfar_game_commands_1_0`, `tfar_plugin_commands_1_0`, `tfar_plugin_settings_1_0`, `tfar_radio_effects_1_0`,
`tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf` and `tfar_geometry`.

## Design rules
These match the 0.9 track, which agent V3 builds in parallel. Mirror its shape where it fits, but never depend on it.
- **Pure state and decisions:** no TeamSpeak calls and no threads. The app (W6) owns one lock per server connection (F-002).
- **Time is injected:** `now: Instant`.
- **DSP objects** are owned per client and created via `ensure_dsp(...)` off the audio thread. The audio path borrows them and allocates nothing.

## `tfar_antennas_1_0`
- `AntennaManager` with `add(records)` (replaces the same net id) and `remove(ids)`.
- `find_connection(sender_pos, sender_range, receiver_pos) -> Option<AntennaConnection { loss }>`, implementing `reference/tfar_1_0/ts/src/antennaManager.cpp:24`: candidates within both ranges, `loss = d_me/antenna_range + d_sender/range`, the minimum below 1.0.
- The original's "null connection loss = 7" becomes `Option` handling. Document how callers map `None`.
- `NetId` parses from the Arma net id string safely. This fixes the non-terminated `strchr` (OF-008); record it as a proposed F-id.

## `tfar_client_state_1_0`
- **`GameData`**, from spec §5 `gameData`:
  - SW/LR maps (`FrequencyEntry` with classname), `speakers`, voice volume, alive, waves, terrain coefficient, global volume, receiving distance multiplier, speaker distance;
  - current data frame, current transmitting radio, tangent pressed, PTT delay.
- **`ClientData`**, keyed by **nickname string** with the TS client id beside it (F-008). It holds every field of spec §5 `clientData`:
  - the vehicle descriptor and velocity;
  - position extrapolation `client_position(now)` (`clientData.cpp:104`);
  - receiving transmission counter and frequencies;
  - the object-interception sample buffer and the per-key effects, filters, HRTF sources and the **two** compressors (direct speech and radio, F-013).
- **Pure queries:** `is_alive(now, current_frame)`, a pure query (F-017); expiry happens only in `sweep_expired(now)`. `effective_distance_to` (`clientData.cpp:125`).
- **`plugin_enabled_for(client, metadata, now)`:** the 10 s positive cache expires 10 s after the metadata was last read (F-016).
- **Remote tangent:** `apply_remote_tangent(...)` returns `TangentEffects { play_start: Option<…>, play_end: Option<…>, receiving_changed }`. A repeated PRESSED is a no-op for sounds and counters (F-015). For RELEASED, the routing is computed before the reset with the ignore flags, as the spec says.
- **The rest:** `apply_remote_volume`, `apply_release_all_tangents`, `apply_position` (an own-client special case), `apply_frequencies` (returns an optional rename request, skipping the three "Error:"/`any` names), `apply_killed`, `apply_speakers`, `next_data_frame`.
- **`GameLinkState`:** in game, `connect`/`disconnect` transitions debounced to 500 ms, and `pluginTimeout` from settings, driven by the server's last-game-message time.

## `tfar_radio_routing_1_0`
- **`is_over_radio`** / **`is_over_local_radio`** (`clientData.cpp:148`, `:267`): intercom first; the local radio with the underwater remap and the antenna fallback; prefer LR when `canUseLRRadio`; the half-duplex block via `current_transmitting_radio`; speakers with the exact-frequency, not-owned-by-sender, z ≥ 0 rules. Returns `Vec<HeardVia { … antenna_loss: Option<f64>, … }>`.
- **Helpers:** `diver_radio_distance`, `distance_underwater` use (from `tfar_geometry`).
- **Mute decisions:** `mute_decision(...)` (`plugin.cpp:68`) works only in serious mode and only without `disableAutomaticMute`. Mute when out of voice range (`voiceVolume + 15`) and unreachable by radio, or on an alive mismatch; never self. A `MuteTracker` with `unmute_all()` returning the tracked set.
- **Serious mode:** `serious_mode_channel(configured_name, channels) -> Option<ChannelId>` supports `ID:<n>` and exact names. The killed-player mute matrix (`CommandProcessor.cpp:552-587`).

## Tests
Table-driven cases per routing rule:
- intercom same vehicle and slot;
- SW, DD remap, LR preference;
- half duplex;
- an antenna better than direct, and an antenna out of range;
- speakers (exact frequency, owned by sender, z < 0).

Plus: F-008 (two nicknames hashing alike no longer collide; a rename keeps the entry correct), F-015, F-016, F-017, extrapolation, the debounce and the mute matrix. Each F-id gets a named test.

## Owned files
`crates/version_1_0/tfar_antennas_1_0/**`, `crates/version_1_0/tfar_client_state_1_0/**`, `crates/version_1_0/tfar_radio_routing_1_0/**`.
Root manifest: one Edit at most (re-read first), reported. If a needed edge is missing from a manifest, for example
`tfar_radio_routing_1_0` → `tfar_game_commands_1_0` for types, add it only if the tier and category rules allow it, run
`cargo xtask verify crate-tiers`, and report it.

## Checks (each alone, with your target dir; logs `target/agent-logs/w3-*.log`)
fmt, clippy `--all-targets -D warnings`, test for the 3 crates; `cargo xtask verify all` (report foreign violations only).

## Report additions
Public API per crate; F-ids pinned, by test name; the lock and ownership contract for W6.

Budget: L (350k tokens). Stop there and report what is done and what is not.
