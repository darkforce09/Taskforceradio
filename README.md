# Taskforceradio

A Rust rewrite of the native parts of Task Force Arrowhead Radio (TFAR), the radio mod for Arma 3, so
that it works with Arma 3 running under Proton and TeamSpeak 3 running as the native Linux Flatpak. The
original mods link the game and TeamSpeak through Win32 named pipes and shared memory, which a native
TeamSpeak plugin cannot reach. Here each mod, 1.0 and 0.9.12, gets a new Arma extension (a Windows x64
DLL with the original name `task_force_radio_pipe`) and a new TeamSpeak plugin (a Linux `.so`); the two
talk over localhost TCP. The mods' SQF and game data stay exactly as released, and the originals are
kept read-only in `reference/` as the specification.

| Mod | Arma extension | TeamSpeak plugin |
|---|---|---|
| TFAR 1.0 | tfar_extension_1_0 | tfar_plugin_1_0 |
| TFAR 0.9.12 | tfar_extension_0_9 | tfar_plugin_0_9 |

## Layout

| Folder | What it holds |
|---|---|
| apps/ | the four deliverables: two TeamSpeak plugins and two Arma extensions |
| crates/ | library crates by category: foundation, audio, teamspeak, game_link, services, interface, version_0_9, version_1_0 |
| tools/ | `cargo xtask` (gates, CI lane, dist, scaffolding), the TeamSpeak mock host, the extension probe, the golden-vector oracle |
| assets/ | the IRCAM HRIR sphere the HRTF crate loads, with its credits |
| `documentation/` | every document: roadmap, decisions, architecture, standards, program records, runbooks |
| `reference/` | the two original mods, byte-identical and read-only ([reference README](/reference/README.md)) |

The full annotated tree is in [CLAUDE.md](/CLAUDE.md#2-directory-atlas); the
[documentation map](/documentation/README.md) lists every document.

## Build and check

Rust 1.99.0 is pinned in `rust-toolchain.toml`; the extensions cross-compile with the mingw linker set in
`.cargo/config.toml`. Run from the repository root:

```bash
cargo build --workspace                                   # plugins and tools for Linux
cargo build -p tfar_extension_1_0 -p tfar_extension_0_9 --target x86_64-pc-windows-gnu
cargo xtask ci ci-local                                   # fmt, clippy, tests, deny, typos, every gate
cargo xtask dist                                          # release artifacts in target/dist/
```

`cargo xtask dist` never copies anything into a mod folder or into TeamSpeak; installing is a manual step
([manual install and test](/documentation/runbooks/manual_install_and_test.md)).

## Status

Milestone M0, the workspace skeleton with its gates and documentation, is in progress. The milestones
M0 to M8 and their acceptance tests are in the [roadmap](/documentation/roadmap.md); the current state is
in [progress](/documentation/program/progress.md). The licence is not decided yet; the upstream mods are
under APL-SA.
