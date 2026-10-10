**Status:** live

# Crate catalogue

Every crate of the workspace (38: 4 apps, 30 libraries, 4 tools), grouped by category: its tier,
the original code it ports, the external crates it may use, the bug fixes it owns and the milestone
that builds it. The workspace edges are the table at the end, read from `cargo metadata`, and the
same edges in the [shared brief](/documentation/program/agent_briefs/shared_brief.md); the laws
that check tiers, edges and firewalls are in
[laws and gates](/documentation/program/laws_and_gates.md). Fix ids point into
[bug fixes](/documentation/program/bug_fixes.md). M0 creates every crate as a skeleton; the
Milestone column names the milestone that fills it.

## Abbreviations

| Short | Meaning |
|---|---|
| T | Tier: 0 with no workspace dependency, else 1 + the highest dependency tier |
| SW, LR, DD | Short-wave (personal) radio, long-range radio, diver radio |
| ILD, ITD | Interaural level difference, interaural time difference |
| HRTF, HRIR | Head-related transfer function, head-related impulse response |
| RBJ | Robert Bristow-Johnson audio-EQ biquad formulas |
| SOS | Second-order sections (a filter as a cascade of biquads) |
| DF2T | Direct Form II Transposed biquad processing |
| TS | TeamSpeak 3 client |
| new | No original code: written from the spec named in the row |

Every "Ports from" path is a file in the reference trees; a `:line` points at the entry function.

## crates/foundation

| Crate | Tier | Ports from (reference file:line) | External crates | Fixes | Milestone |
|---|---|---|---|---|---|
| `tfar_geometry` | 0 | `reference/tfar_1_0/ts/src/datatypes.cpp:105` (vectors, angles, `distanceUnderwater`), `reference/tfar_1_0/ts/src/datatypes.hpp`, `reference/tfar_0_9_12/ts/src/plugin.cpp:608` (angle helpers) | `glam` | F-009 | M1 |
| `tfar_arma_values` | 1 | `reference/tfar_0_9_12/ts/src/plugin.cpp:801` (getline split), `reference/tfar_0_9_12/ts/src/plugin.cpp:112` (Arma numbers), `reference/tfar_0_9_12/ts/src/plugin.cpp:1515` (frequency lists), `reference/tfar_0_9_12/ts/src/plugin.cpp:967` (vehicle descriptor), `reference/tfar_1_0/ts/src/helpers.cpp:307` (frequency lists with classname), `reference/tfar_1_0/ts/src/plugin.cpp:116` (subtypes) | — | F-010 | M1 |

## crates/audio

| Crate | Tier | Ports from (reference file:line) | External crates | Fixes | Milestone |
|---|---|---|---|---|---|
| `tfar_sample_buffer` | 0 | `reference/tfar_1_0/ts/src/SampleBuffer.hpp`, `reference/tfar_1_0/ts/src/SampleBuffer.cpp:24`, `reference/tfar_0_9_12/ts/src/plugin.cpp:603` (gain), `reference/tfar_0_9_12/ts/src/plugin.cpp:2831` (channel adapter) | — | F-004, F-006 | M1 |
| `tfar_dsp_filters` | 1 | `reference/tfar_1_0/ts/src/helpers.hpp:152` (`processFilterStereo`), `reference/tfar_0_9_12/ts/src/plugin.cpp:232` (phone, speaker, cant-speak and vehicle filter setups), `reference/tfar_1_0/ts/include/dspfilters/Butterworth.h`, `reference/tfar_1_0/ts/include/dspfilters/RBJ.h` | `sci-rs`, `biquad` | — | M4 |
| `tfar_dynamics` | 1 | `reference/tfar_1_0/ts/include/simpleSource/SimpleComp.h`, `reference/tfar_1_0/ts/include/simpleSource/SimpleCompProcess.inl`, `reference/tfar_1_0/ts/src/RadioEffect.hpp:286` (`processCompressor`) | — | — | M4 |
| `tfar_hrtf` | 1 | new: binaural rendering on the `hrtf` crate with the IRCAM sphere in `assets/hrir/`; replaces clunk (`reference/tfar_1_0/ts/src/Clunk.h`, `reference/tfar_0_9_12/ts/src/Clunk.h`, `reference/tfar_1_0/ts/include/clunk/hrtf.cpp:95`), whose KEMAR filter is disabled | `hrtf` | F-001 | M4 |
| `tfar_spatial_audio` | 1 | `reference/tfar_1_0/ts/src/helpers.cpp:30` (X3DAudio curve and cone), `reference/tfar_1_0/ts/src/helpers.cpp:12` (simple ILD), `reference/tfar_0_9_12/ts/src/plugin.cpp:608` (ILD) | `glam` | F-012 | M4 |
| `tfar_playback_mixer` | 1 | `reference/tfar_1_0/ts/src/PlaybackHandler.cpp:36` (mixed playback), `reference/tfar_1_0/ts/src/PlaybackHandler.cpp:147` (positional playback), `reference/tfar_0_9_12/ts/src/plugin.cpp:711` (WAV playback), `reference/tfar_0_9_12/ts/src/plugin.cpp:2600` (mix) | `hound`, `rtrb` | F-005 | M4 |

## crates/teamspeak

| Crate | Tier | Ports from (reference file:line) | External crates | Fixes | Milestone |
|---|---|---|---|---|---|
| `tfar_ts3_host` | 0 | `reference/tfar_1_0/ts/src/Teamspeak.cpp:778` (metadata exports), `reference/tfar_1_0/ts/src/Teamspeak.hpp`, `reference/tfar_0_9_12/ts/src/plugin.cpp:2128` (exports), `reference/tfar_1_0/ts/include/ts3_functions.h` | `ts3plugin-sys`, `libc`, `tracing`, `tracing-subscriber`, `parking_lot` | F-018 | M2 |
| `tfar_ts3_settings_db` | 0 | `reference/tfar_0_9_12/ts/src/plugin.cpp:2248` (PTT delay query), `reference/tfar_1_0/ts/src/plugin.cpp:344` | `rusqlite` (`bundled`) | — | M2 |

## crates/game_link

| Crate | Tier | Ports from (reference file:line) | External crates | Fixes | Milestone |
|---|---|---|---|---|---|
| `tfar_game_link_protocol` | 0 | new: [game link protocol](/documentation/architecture/game_link_protocol.md); replaces `reference/tfar_1_0/ts/src/SharedMemoryHandler.hpp` and the 0.9 pipe | `postcard`, `serde` | — | M3 |
| `tfar_game_link_server` | 1 | `reference/tfar_1_0/ts/src/SharedMemoryHandler.cpp:14` (queue), `reference/tfar_1_0/ts/src/plugin.cpp:166` (PipeThread), `reference/tfar_0_9_12/ts/src/plugin.cpp:2022` (PipeThread) | `postcard`, `parking_lot`, `crossbeam-channel` | F-007 | M3 |
| `tfar_game_link_client` | 1 | `reference/tfar_1_0/extensions/task_force_radio_pipe/SharedMemoryTransfer.cpp:250`, `reference/tfar_0_9_12/arma3/@task_force_radio/task_force_radio_pipe/task_force_radio_pipe.cpp:53` | `postcard`, `parking_lot` | — | M3 |
| `tfar_arma_extension_abi` | 0 | `reference/tfar_1_0/extensions/task_force_radio_pipe/task_force_radio_pipe.cpp:9` (`RVExtension`), `reference/tfar_1_0/extensions/task_force_radio_pipe/stdafx.h` (`-tfdebug`), `reference/tfar_1_0/ts/src/Teamspeak.cpp:207` (focus check) | `windows-sys` | — | M3 |

## crates/services

| Crate | Tier | Ports from (reference file:line) | External crates | Fixes | Milestone |
|---|---|---|---|---|---|
| `tfar_web_requests` | 0 | `reference/tfar_0_9_12/ts/src/plugin.cpp:124` (Piwik), `reference/tfar_0_9_12/ts/src/plugin.cpp:155` (update check), `reference/tfar_1_0/ts/src/task_force_radio.cpp:212` (Piwik), `reference/tfar_1_0/ts/src/task_force_radio.cpp:336` (update check) | `ureq` (rustls), `url`, `version-compare` | F-026, F-030 | M5 |

## crates/interface

| Crate | Tier | Ports from (reference file:line) | External crates | Fixes | Milestone |
|---|---|---|---|---|---|
| `tfar_debug_window` | 0 | `reference/tfar_1_0/ts/src/DebugUI.cpp`, `reference/tfar_1_0/ts/src/TextRenderer.cpp` | `eframe`/`egui` (glow, x11), `winit` (x11) | — | M5 |

## crates/version_0_9

| Crate | Tier | Ports from (reference file:line) | External crates | Fixes | Milestone |
|---|---|---|---|---|---|
| `tfar_game_commands_0_9` | 2 | `reference/tfar_0_9_12/ts/src/plugin.cpp:1769` (`processGameCommand`), `reference/tfar_0_9_12/ts/src/plugin.cpp:1534` (`ts_info`) | — | F-003, F-020, F-021, F-027, F-028 | M6 |
| `tfar_plugin_commands_0_9` | 2 | `reference/tfar_0_9_12/ts/src/plugin.cpp:3071` (`processPluginCommand`), `reference/tfar_0_9_12/ts/src/plugin.cpp:1300` (metadata block) | — | — | M6 |
| `tfar_radio_effects_0_9` | 2 | `reference/tfar_0_9_12/ts/src/RadioEffect.h` | `fastrand` | F-019 | M6 |
| `tfar_client_state_0_9` | 3 | `reference/tfar_0_9_12/ts/src/plugin.cpp:192` (`CLIENT_DATA`), `reference/tfar_0_9_12/ts/src/plugin.cpp:370` (server data), `reference/tfar_0_9_12/ts/src/plugin.cpp:1938` (expiry), `reference/tfar_0_9_12/ts/src/plugin.cpp:1418` (serious mode) | `parking_lot`, `arc-swap` | F-002, F-022, F-024, F-032, F-033 | M6 |
| `tfar_radio_routing_0_9` | 4 | `reference/tfar_0_9_12/ts/src/plugin.cpp:989` (`isOverLocalRadio`), `reference/tfar_0_9_12/ts/src/plugin.cpp:1091` (`isOverRadio`), `reference/tfar_0_9_12/ts/src/plugin.cpp:1188` (mute status), `reference/tfar_0_9_12/ts/src/plugin.cpp:1356` (dead players) | — | F-023, F-025 | M6 |
| `tfar_voice_pipeline_0_9` | 5 | `reference/tfar_0_9_12/ts/src/plugin.cpp:2653` (stereo function), `reference/tfar_0_9_12/ts/src/plugin.cpp:2852` (captured voice), `reference/tfar_0_9_12/ts/src/plugin.cpp:2476` (`processRadioEffect`), `reference/tfar_0_9_12/ts/src/plugin.cpp:698` (`volumeFromDistance`) | — | F-051 | M6 |

## crates/version_1_0

| Crate | Tier | Ports from (reference file:line) | External crates | Fixes | Milestone |
|---|---|---|---|---|---|
| `tfar_plugin_settings_1_0` | 0 | `reference/tfar_1_0/ts/src/settings.hpp` | `parking_lot` | F-031 | M7 |
| `tfar_antennas_1_0` | 2 | `reference/tfar_1_0/ts/src/antennaManager.cpp:24` (`findConnection`), `reference/tfar_1_0/ts/src/antennaManager.h` | — | F-041 | M7 |
| `tfar_game_commands_1_0` | 2 | `reference/tfar_1_0/ts/src/CommandProcessor.cpp:72` (sync), `reference/tfar_1_0/ts/src/CommandProcessor.cpp:224` (async), `reference/tfar_1_0/ts/src/task_force_radio.hpp:47` (FREQ) | — | F-003, F-029 | M7 |
| `tfar_plugin_commands_1_0` | 2 | `reference/tfar_1_0/ts/src/plugin.cpp:1026` (`processPluginCommand`), `reference/tfar_1_0/ts/src/Teamspeak.cpp:644` (metadata block) | — | F-014 | M7 |
| `tfar_radio_effects_1_0` | 2 | `reference/tfar_1_0/ts/src/RadioEffect.hpp` | `fastrand` | F-011 | M7 |
| `tfar_client_state_1_0` | 3 | `reference/tfar_1_0/ts/src/clientData.cpp`, `reference/tfar_1_0/ts/src/serverData.cpp`, `reference/tfar_1_0/ts/src/task_force_radio.cpp`, `reference/tfar_1_0/ts/src/plugin.cpp:879` (`processTangentPress`) | `parking_lot`, `arc-swap` | F-002, F-008, F-015, F-016, F-017, F-042–F-047 | M7 |
| `tfar_radio_routing_1_0` | 4 | `reference/tfar_1_0/ts/src/clientData.cpp:148` (`isOverLocalRadio`), `reference/tfar_1_0/ts/src/clientData.cpp:267` (`isOverRadio`), `reference/tfar_1_0/ts/src/plugin.cpp:68` (mute status) | — | — | M7 |
| `tfar_voice_pipeline_1_0` | 5 | `reference/tfar_1_0/ts/src/plugin.cpp:455` (`processVoiceData`), `reference/tfar_1_0/ts/src/plugin.cpp:789` (captured voice), `reference/tfar_1_0/ts/src/helpers.hpp:122` (`volumeAttenuation`) | — | F-013, F-061–F-063 | M7 |

## apps

Each app is a `cdylib` + `rlib` whose `[lib] name` is the crate name; `cargo xtask dist` gives the
files their installed names. Apps sit at tier 6 (plugins) or 2 (extensions).

| Crate | Tier | Ports from (reference file:line) | External crates | Fixes | Milestone |
|---|---|---|---|---|---|
| `tfar_plugin_0_9` | 6 | `reference/tfar_0_9_12/ts/src/plugin.cpp:2217` (init), `reference/tfar_0_9_12/ts/src/plugin.cpp:1977` (ServiceThread), `reference/tfar_0_9_12/ts/src/plugin.cpp:2398` (TS events) | `tracing`, `profiling`, `tracy-client` (feature `tracy`), `parking_lot`, `crossbeam-channel` | F-071 | M0 skeleton, M2 exports, M6 full |
| `tfar_plugin_1_0` | 6 | `reference/tfar_1_0/ts/src/plugin.cpp:289` (init), `reference/tfar_1_0/ts/src/plugin.cpp:133` (ServiceThread), `reference/tfar_1_0/ts/src/Teamspeak.cpp:838` (`/tfar` commands) | `tracing`, `profiling`, `tracy-client` (feature `tracy`), `parking_lot`, `crossbeam-channel` | F-081, F-082 | M0 skeleton, M2 exports, M7 full |
| `tfar_extension_0_9` | 2 | `reference/tfar_0_9_12/arma3/@task_force_radio/task_force_radio_pipe/task_force_radio_pipe.cpp:53` | — | — | M0 skeleton, M3 transport, M6 full |
| `tfar_extension_1_0` | 2 | `reference/tfar_1_0/extensions/task_force_radio_pipe/task_force_radio_pipe.cpp:9`, `reference/tfar_1_0/extensions/task_force_radio_pipe/SharedMemoryTransfer.cpp:250` | — | — | M0 skeleton, M3 transport, M7 full |

## tools

| Crate | Tier | Ports from (reference file:line) | External crates | Fixes | Milestone |
|---|---|---|---|---|---|
| `xtask` | 0 | new: [laws and gates](/documentation/program/laws_and_gates.md) | `clap`, `xshell`, `cargo_metadata`, `toml_edit`, `sha2`, `walkdir`, `ignore`, `anyhow` | — | M0 |
| `tfar_ts3_mock` | 2 | new: a fake `TS3Functions` table after `reference/tfar_1_0/ts/include/ts3_functions.h` | `libloading`, `ts3plugin-sys`, `libc`, `parking_lot` | — | M2 |
| `tfar_extension_probe` | 0 | new: loads the extension DLL and calls `RVExtension` like Arma | `libloading`, `anyhow` | — | M3 |
| `tfar_golden_vectors` | 0 | live oracles compiled from `reference/tfar_1_0/ts/include/dspfilters/` and `reference/tfar_1_0/ts/include/simpleSource/`, plus a vendored FAudio F3DAudio.c, compared against the audio crates in its tests | `cc` (build script) | — | M4 |

## Workspace edges

Every workspace dependency, read from `cargo metadata` on 2026-10-03 at 0ced2f5; crates with
none are left out. Dev-dependencies serve tests only and are exempt from tier order.

| Crate | Depends on | Dev-depends on |
|---|---|---|
| `tfar_antennas_1_0` | `tfar_arma_values`, `tfar_geometry` | — |
| `tfar_arma_values` | `tfar_geometry` | — |
| `tfar_client_state_0_9` | `tfar_dsp_filters`, `tfar_dynamics`, `tfar_game_commands_0_9`, `tfar_geometry`, `tfar_hrtf`, `tfar_radio_effects_0_9` | — |
| `tfar_client_state_1_0` | `tfar_arma_values`, `tfar_dsp_filters`, `tfar_dynamics`, `tfar_game_commands_1_0`, `tfar_geometry`, `tfar_hrtf`, `tfar_plugin_commands_1_0`, `tfar_plugin_settings_1_0`, `tfar_radio_effects_1_0`, `tfar_sample_buffer` | — |
| `tfar_dsp_filters` | `tfar_sample_buffer` | — |
| `tfar_dynamics` | `tfar_sample_buffer` | — |
| `tfar_extension_0_9` | `tfar_arma_extension_abi`, `tfar_game_link_client` | `tfar_game_link_protocol`, `tfar_game_link_server` |
| `tfar_extension_1_0` | `tfar_arma_extension_abi`, `tfar_game_link_client` | `tfar_game_link_server` |
| `tfar_game_commands_0_9` | `tfar_arma_values`, `tfar_geometry` | — |
| `tfar_game_commands_1_0` | `tfar_arma_values`, `tfar_geometry`, `tfar_plugin_settings_1_0` | — |
| `tfar_game_link_client` | `tfar_game_link_protocol` | — |
| `tfar_game_link_server` | `tfar_game_link_protocol` | — |
| `tfar_golden_vectors` | — | `tfar_dsp_filters`, `tfar_dynamics`, `tfar_geometry`, `tfar_sample_buffer`, `tfar_spatial_audio` |
| `tfar_hrtf` | `tfar_geometry`, `tfar_sample_buffer` | — |
| `tfar_playback_mixer` | `tfar_sample_buffer` | — |
| `tfar_plugin_0_9` | `tfar_arma_values`, `tfar_client_state_0_9`, `tfar_dsp_filters`, `tfar_game_commands_0_9`, `tfar_game_link_server`, `tfar_geometry`, `tfar_hrtf`, `tfar_playback_mixer`, `tfar_plugin_commands_0_9`, `tfar_radio_routing_0_9`, `tfar_sample_buffer`, `tfar_spatial_audio`, `tfar_ts3_host`, `tfar_ts3_settings_db`, `tfar_voice_pipeline_0_9`, `tfar_web_requests` | `tfar_game_link_client`, `tfar_ts3_mock` |
| `tfar_plugin_1_0` | `tfar_antennas_1_0`, `tfar_arma_values`, `tfar_client_state_1_0`, `tfar_debug_window`, `tfar_game_commands_1_0`, `tfar_game_link_server`, `tfar_hrtf`, `tfar_playback_mixer`, `tfar_plugin_commands_1_0`, `tfar_plugin_settings_1_0`, `tfar_radio_routing_1_0`, `tfar_sample_buffer`, `tfar_ts3_host`, `tfar_ts3_settings_db`, `tfar_voice_pipeline_1_0`, `tfar_web_requests` | `tfar_game_link_client`, `tfar_ts3_mock` |
| `tfar_plugin_commands_0_9` | `tfar_arma_values` | — |
| `tfar_plugin_commands_1_0` | `tfar_arma_values` | — |
| `tfar_radio_effects_0_9` | `tfar_arma_values`, `tfar_dsp_filters`, `tfar_sample_buffer` | — |
| `tfar_radio_effects_1_0` | `tfar_arma_values`, `tfar_dsp_filters`, `tfar_sample_buffer` | — |
| `tfar_radio_routing_0_9` | `tfar_client_state_0_9`, `tfar_geometry` | — |
| `tfar_radio_routing_1_0` | `tfar_antennas_1_0`, `tfar_arma_values`, `tfar_client_state_1_0`, `tfar_geometry`, `tfar_plugin_settings_1_0` | — |
| `tfar_spatial_audio` | `tfar_geometry`, `tfar_sample_buffer` | — |
| `tfar_ts3_mock` | `tfar_game_link_client`, `tfar_game_link_protocol`, `tfar_ts3_host` | — |
| `tfar_voice_pipeline_0_9` | `tfar_client_state_0_9`, `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`, `tfar_playback_mixer`, `tfar_radio_effects_0_9`, `tfar_radio_routing_0_9`, `tfar_sample_buffer`, `tfar_spatial_audio` | — |
| `tfar_voice_pipeline_1_0` | `tfar_antennas_1_0`, `tfar_client_state_1_0`, `tfar_dsp_filters`, `tfar_dynamics`, `tfar_geometry`, `tfar_hrtf`, `tfar_playback_mixer`, `tfar_plugin_settings_1_0`, `tfar_radio_effects_1_0`, `tfar_radio_routing_1_0`, `tfar_sample_buffer`, `tfar_spatial_audio` | — |
