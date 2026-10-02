# Library crates

Every library the four deliverables share, one folder per category. Each crate lives at
crates/<category>/<crate>/, its package name equals its folder name, and its manifest records the
category and tier under `[package.metadata.layout]`.

## Contents

```text
crates/
├── audio/        sample buffer, filters, compressor, HRTF, distance and cone gains, radio sound mixer
├── foundation/   geometry and the Arma value parsers; depends on nothing in the workspace
├── game_link/    the localhost TCP link between extensions and plugins, and the Arma extension exports
├── interface/    the 1.0 debug window
├── services/     the update check and Piwik calls
├── teamspeak/    the TeamSpeak 3 SDK export layer and the settings database reader
├── version_0_9/  the behaviour of the 0.9.12 mod, from game commands to the voice pipeline
└── version_1_0/  the behaviour of the 1.0 mod, from settings and antennas to the voice pipeline
```

## How it works

The categories form layers. Foundation sits at the bottom; audio, teamspeak and game_link build on
it; services and interface are leaves beside them; the two version tracks compose all of these into
the behaviour of one mod each. The apps in `apps/` link one version track plus the shared layers.

| Category | Crates | What it holds |
|---|---|---|
| `crates/foundation` | `tfar_geometry`, `tfar_arma_values` | positions, directions, Arma value parsing |
| `crates/audio` | `tfar_sample_buffer`, `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`, `tfar_spatial_audio`, `tfar_playback_mixer` | the audio building blocks both versions share |
| `crates/teamspeak` | `tfar_ts3_host`, `tfar_ts3_settings_db` | the boundary to the TeamSpeak 3 client |
| `crates/game_link` | `tfar_game_link_protocol`, `tfar_game_link_server`, `tfar_game_link_client`, `tfar_arma_extension_abi` | the extension-to-plugin link and the Arma exports |
| `crates/services` | `tfar_web_requests` | outbound HTTPS: update check, Piwik |
| `crates/interface` | `tfar_debug_window` | the 1.0 debug window |
| `crates/version_0_9` | six `_0_9` crates | the 0.9.12 behaviour |
| `crates/version_1_0` | eight `_1_0` crates | the 1.0 behaviour |

Edge matrix: a row may depend on the columns marked `x`. Dev-dependencies are exempt from tier
order but never point at apps.

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
plugin apps           x        x     x         x          x        x     own version only
extension apps        x                        x
tools                 x        x     x         x          x        x       x    x

* only tfar_game_link_protocol
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
cargo build --workspace   # every crate and app for the Linux host
cargo test --workspace    # every unit test
```

## Boundaries

- Depends on: the external crates pinned in the root `Cargo.toml`.
- Used by: the four apps in `apps/` and the tools in `tools/`.
- Rules: the edge matrix above, the tier rule and the external-crate firewalls each category README
  names (`cargo xtask verify crate-tiers`); `unsafe` only in `tfar_ts3_host` and
  `tfar_arma_extension_abi` among the libraries (`cargo xtask verify unsafe-allowlist`).

## Related documentation

- [Conventions survey](/documentation/program/research/conventions_survey.md) — where the layout,
  tier metadata and crate anatomy come from.
- [Library survey](/documentation/program/research/library_survey.md) — the external crates chosen
  for each need.
