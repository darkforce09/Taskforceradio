**Status:** live

# Workspace layout

How the Cargo workspace is arranged: the tree, the crate categories, the rule that decides which
folder a new crate goes in, and how the crates depend on each other. The laws that check this layout
are in [laws and gates](/documentation/program/laws_and_gates.md); each crate's role, sources and
milestone are in the [crate catalogue](/documentation/program/crate_catalogue.md).

## Tree

```text
Taskforceradio/
├── Cargo.toml  Cargo.lock  rust-toolchain.toml  rustfmt.toml  clippy.toml  deny.toml  typos.toml
├── .cargo/config.toml   xtask alias; windows-gnu linker x86_64-w64-mingw32-gcc, +crt-static
├── .editorconfig  .gitignore  .gitattributes  README.md  CLAUDE.md  AGENTS.md -> CLAUDE.md
├── crates/
│   ├── foundation/   tfar_geometry (glam domain helpers)  tfar_arma_values
│   ├── audio/        tfar_sample_buffer  tfar_dsp_filters  tfar_dynamics  tfar_hrtf
│   │                 tfar_spatial_audio (distance curve + cone)  tfar_playback_mixer (hound)
│   ├── teamspeak/    tfar_ts3_host (export layer on ts3plugin-sys)  tfar_ts3_settings_db
│   ├── game_link/    tfar_game_link_protocol  tfar_game_link_server  tfar_game_link_client
│   │                 tfar_arma_extension_abi
│   ├── services/     tfar_web_requests          (update check, Piwik; ureq + rustls)
│   ├── interface/    tfar_debug_window          (egui window, X11)
│   ├── version_0_9/
│   │   ├── tfar_plugin_0_9/     cdylib: TS exports, threads, TS events, PTT/VAD, metadata, serious mode
│   │   ├── tfar_extension_0_9/  cdylib DLL: every call is a sync round trip
│   │   └── …                    tfar_game_commands_0_9  tfar_plugin_commands_0_9  tfar_radio_effects_0_9
│   │                            tfar_client_state_0_9  tfar_radio_routing_0_9  tfar_voice_pipeline_0_9
│   └── version_1_0/
│       ├── tfar_plugin_1_0/     same for 1.0 (+ async queue, antennas, SETCFG, focus-aware channel move)
│       ├── tfar_extension_1_0/  cdylib DLL: ~ async, DFRAME -> NEEDCFG/OK, MISSIONEND, FOCUS frames
│       └── …                    tfar_game_commands_1_0  tfar_plugin_commands_1_0  tfar_radio_effects_1_0
│                                tfar_client_state_1_0  tfar_radio_routing_1_0  tfar_voice_pipeline_1_0
│                                tfar_antennas_1_0  tfar_plugin_settings_1_0

├── tools/
│   ├── xtask/                    verify · ci · dist · new-crate · reference-manifest
│   ├── tfar_ts3_mock/            dlopens a plugin .so, fake TS3Functions, scripted scenarios
│   ├── tfar_extension_probe/     Windows exe: LoadLibrary + RVExtension (run under Proton's wine)
│   └── tfar_golden_vectors/      cc-built reference C++ and F3DAudio, live oracles in its tests
├── assets/hrir/     IRCAM HRIR sphere (.bin) + CREDITS.md
├── documentation/   roadmap, program records, architecture, standards, decisions, runbooks
└── reference/       README.md + MANIFEST.sha256 (written by xtask); the two mod trees, byte-identical
```

38 crates: 4 deliverables (the apps), 30 libraries, 4 tools. Members are globbed as `"crates/*/*"`
and `"tools/*"`, so a crate is a member by being in the right folder. There is no apps folder: each
deliverable sits in its version category beside the libraries it links, and the gates know the four
by name.

## Categories

| Folder | Holds | Example |
|---|---|---|
| crates/foundation | Plain data and maths with no audio, TeamSpeak or link knowledge | `tfar_geometry` |
| crates/audio | Sample buffers and DSP that any voice chain can use | `tfar_dsp_filters` |
| crates/teamspeak | The TeamSpeak plugin SDK boundary and TeamSpeak's own files | `tfar_ts3_host` |
| crates/game_link | The extension ↔ plugin transport and the Arma extension ABI | `tfar_game_link_server` |
| crates/services | Calls to the outside world (HTTP) | `tfar_web_requests` |
| crates/interface | Windows a person looks at | `tfar_debug_window` |
| crates/version_0_9 | Behaviour that exists only in the 0.9.12 mod, and its two deliverables | `tfar_radio_routing_0_9`, `tfar_plugin_0_9` |
| crates/version_1_0 | Behaviour that exists only in the 1.0 mod, and its two deliverables | `tfar_antennas_1_0`, `tfar_extension_1_0` |
| tools | Repository tooling and test harnesses, never shipped | `tfar_ts3_mock` |

### Which folder does a new crate go in

Ask in this order and stop at the first yes:
1. Is it shipped to the owner as a `.so` or `.dll`? → its version category, crates/version_0_9 or
   crates/version_1_0, named in the deliverables list of the gates (thin export glue and wiring).
2. Is it only for building, checking or testing the repository? → tools.
3. Does it hold behaviour only one mod version has (its command set, its routing rules, its
   effects)? → crates/version_0_9 or crates/version_1_0. Never both: shared behaviour moves down.
4. Does it touch the TeamSpeak SDK or TeamSpeak's files? → crates/teamspeak.
5. Does it carry game ↔ plugin traffic or the Arma ABI? → crates/game_link.
6. Does it process samples? → crates/audio.
7. Does it call a network service? → crates/services. Does it open a window? → crates/interface.
8. Otherwise it is plain data or maths → crates/foundation.

Then scaffold it with `cargo xtask new-crate <category> <name>`, which writes the manifest with its
layout metadata and computed tier, the READMEs, `lib.rs`, `prelude.rs`, `error.rs` and a first
module with its sibling test.

## Dependency diagram

Arrows point from a crate to what it depends on; tiers run from 0 at the bottom up to 6. The full
edge list, read from `cargo metadata`, is in the
[crate catalogue](/documentation/program/crate_catalogue.md#workspace-edges).

```text
 tier 6   tfar_plugin_0_9                                    tfar_plugin_1_0
             |  uses version_0_9 + host + link server          |  uses version_1_0 + host + link
             |  + web requests + mixer + foundation            |  server + web + debug window + ...
             v                                                 v
 tier 5   tfar_voice_pipeline_0_9                            tfar_voice_pipeline_1_0
 tier 4   tfar_radio_routing_0_9                             tfar_radio_routing_1_0 --> tfar_antennas_1_0
 tier 3   tfar_client_state_0_9                              tfar_client_state_1_0
 tier 2   game_commands_0_9  plugin_commands_0_9             game_commands_1_0  plugin_commands_1_0
          radio_effects_0_9                                  radio_effects_1_0  antennas_1_0
             |                                                 |      (+ plugin_settings_1_0, tier 0)
             |         version_0_9 never <---X---> version_1_0 |
             v                                                 v
 tier 1   tfar_dsp_filters  tfar_dynamics  tfar_hrtf  tfar_spatial_audio  tfar_playback_mixer
          tfar_arma_values  tfar_game_link_server  tfar_game_link_client
 tier 0   tfar_sample_buffer  tfar_geometry  tfar_ts3_host  tfar_ts3_settings_db
          tfar_game_link_protocol  tfar_arma_extension_abi  tfar_web_requests  tfar_debug_window

 tier 2   tfar_extension_0_9, tfar_extension_1_0 --> tfar_arma_extension_abi, tfar_game_link_client
 tools    tfar_ts3_mock (tier 2) --> tfar_ts3_host, tfar_game_link_protocol, tfar_game_link_client
          xtask, tfar_extension_probe, tfar_golden_vectors (tier 0; the oracle dev-depends on the
          audio crates it checks)
```
