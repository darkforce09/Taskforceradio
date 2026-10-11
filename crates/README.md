# Crates

Every product crate, one folder per category: the shared libraries and, per mod version, its
libraries plus its two deliverables (the TeamSpeak plugin and the Arma extension). A crate lives at
`crates/<category>/<crate>/`, its package name equals its folder name, and its manifest records the
category and tier under `[package.metadata.layout]`.

## How it works

The categories form layers. Foundation sits at the bottom; audio, teamspeak and game_link build on
it; services and interface are leaves beside them; the two version tracks compose all of these into
the behaviour of one mod each. Each version folder also holds that mod's two deliverables: the
plugin links its version track plus the shared layers, the extension only foundation and game_link.
Arma calls the extension, which forwards over localhost TCP to the plugin TeamSpeak has loaded (0.9
on port 47810, 1.0 on 47820).

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

**Where does new code go?** Into an existing crate first. Code that knows a mod version goes in
that version's folder; code both versions share goes in the shared category it belongs to; code
that only runs in tests or by hand goes in `tools/`. A new crate needs a real boundary (see
[CLAUDE.md](/CLAUDE.md)).

## Getting started

```bash
cargo build --workspace   # every crate for the Linux host, the two plugins included
cargo test --workspace    # every test
```

## Boundaries

The category edge matrix, the tiers, the external-crate firewalls and the unsafe allow-list are in
the [crate boundary rules](/documentation/standards/crate_boundary_rules.md), enforced by
`cargo xtask verify crate-tiers`, `dependency-firewalls` and `unsafe-allowlist`.
