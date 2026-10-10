# Crates

Every crate of the product, one folder per category: the shared libraries and, per mod version,
its libraries plus its two deliverables (the TeamSpeak plugin and the Arma extension). Each crate
lives at crates/<category>/<crate>/, its package name equals its folder name, and its manifest
records the category and tier under `[package.metadata.layout]`.

## Contents

```text
crates/
├── audio/        sample buffer, filters, compressor, HRTF, distance and cone gains, radio sound mixer
├── foundation/   geometry and the Arma value parsers; depends on nothing in the workspace
├── game_link/    the localhost TCP link between extensions and plugins, and the Arma extension exports
├── interface/    the 1.0 debug window
├── services/     the update check and Piwik calls
├── teamspeak/    the TeamSpeak 3 SDK export layer and the settings database reader
├── version_0_9/  the 0.9.12 mod: its plugin and extension, and its libraries from game commands to voice
└── version_1_0/  the 1.0 mod: its plugin and extension, and its libraries from settings to voice
```

## How it works

The categories form layers. Foundation sits at the bottom; audio, teamspeak and game_link build on
it; services and interface are leaves beside them; the two version tracks compose all of these into
the behaviour of one mod each. Each version folder also holds that mod's two deliverables (apps):
the plugin links its version track plus the shared layers, the extension only foundation and
game_link. Arma calls the extension, which forwards over localhost TCP to the plugin TeamSpeak has
loaded (0.9 on port 47810, 1.0 on 47820).

| Category | Crates | What it holds |
|---|---|---|
| `crates/foundation` | `tfar_geometry`, `tfar_arma_values` | positions, directions, Arma value parsing |
| `crates/audio` | `tfar_sample_buffer`, `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`, `tfar_spatial_audio`, `tfar_playback_mixer` | the audio building blocks both versions share |
| `crates/teamspeak` | `tfar_ts3_host`, `tfar_ts3_settings_db` | the boundary to the TeamSpeak 3 client |
| `crates/game_link` | `tfar_game_link_protocol`, `tfar_game_link_server`, `tfar_game_link_client`, `tfar_arma_extension_abi` | the extension-to-plugin link and the Arma exports |
| `crates/services` | `tfar_web_requests` | outbound HTTPS: update check, Piwik |
| `crates/interface` | `tfar_debug_window` | the 1.0 debug window |
| `crates/version_0_9` | six `_0_9` libraries, `tfar_plugin_0_9`, `tfar_extension_0_9` | the 0.9.12 behaviour and deliverables |
| `crates/version_1_0` | eight `_1_0` libraries, `tfar_plugin_1_0`, `tfar_extension_1_0` | the 1.0 behaviour and deliverables |

Edge matrix: a row may depend on the columns marked `x`. The deliverables rows are keyed by crate name, not by folder. Dev-dependencies are exempt from tier
order but never point at a deliverable, and nothing depends on one.

```text
                  foundation audio teamspeak game_link services interface v0_9 v1_0
foundation            x
audio                 x        x
teamspeak             x              x
game_link             x                        x
services              x
interface             x
version_0_9           x        x     x      protocol*     x        x       x
version_1_0           x        x     x      protocol*     x        x            x
tfar_plugin_X         x        x     x         x          x        x     own version only
tfar_extension_X      x                        x
tools                 x        x     x         x          x        x       x    x

* only tfar_game_link_protocol; no row may depend on tfar_plugin_X or tfar_extension_X
```

The tier of a crate is 0 when it has no workspace dependency, else one above its highest workspace
dependency, so every edge points strictly down.

**Where does a new crate go?** Pick the lowest category whose allowed edges cover every workspace
crate it needs. Code that knows a mod version goes in that version's folder; code both versions
share goes in the shared category it belongs to, never in a version folder; code that only runs in
tests or by hand goes in `tools/`. If no category fits, the edge matrix changes first, by decision.

## Getting started

Run from the repository root:

```bash
cargo build --workspace   # every crate for the Linux host, the two plugins included
cargo test --workspace    # every unit test
```

## Boundaries

- Depends on: the external crates pinned in the root `Cargo.toml`.
- Used by: TeamSpeak 3 and Arma 3, which load the four deliverables; the tools in `tools/`.
- Rules: the edge matrix above, the tier rule and the external-crate firewalls each category README
  names (`cargo xtask verify crate-tiers`); `unsafe` only in `tfar_ts3_host`,
  `tfar_arma_extension_abi` and the four deliverables (export glue) (`cargo xtask verify unsafe-allowlist`).

## Related documentation

- [Conventions survey](/documentation/program/research/conventions_survey.md) — where the layout,
  tier metadata and crate anatomy come from.
- [Library survey](/documentation/program/research/library_survey.md) — the external crates chosen
  for each need.
