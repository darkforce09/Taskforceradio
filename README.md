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
| TFAR 1.0 | `tfar_extension_1_0` | `tfar_plugin_1_0` |
| TFAR 0.9.12 | `tfar_extension_0_9` | `tfar_plugin_0_9` |

## Layout

| Folder | What it holds |
|---|---|
| `crates/` | Every product crate by category: foundation, audio, teamspeak, game_link, services, interface, and version_0_9 and version_1_0, each holding its mod's plugin, extension and voice logic |
| `tools/` | `cargo xtask` (gates, CI lane, dist), the TeamSpeak mock host, the extension probe, the golden-vector oracle |
| `assets/` | The IRCAM HRIR sphere the HRTF loads, with its credits |
| `documentation/` | Architecture, decisions, behaviour specs of the originals, standards, glossary, runbook ([map](/documentation/README.md)) |
| `reference/` | The two original mods, byte-identical and read-only ([reference README](/reference/README.md)) |

## Build and check

Rust 1.99.0 is pinned in `rust-toolchain.toml`; the extensions cross-compile with the mingw linker set in
`.cargo/config.toml`. Run from the repository root:

```bash
cargo build --workspace                                   # plugins and tools for Linux
cargo build -p tfar_extension_1_0 -p tfar_extension_0_9 --target x86_64-pc-windows-gnu
cargo xtask ci ci-local                                   # the CI: fmt, clippy, tests, deny, typos, every gate
cargo xtask dist                                          # release artifacts in target/dist/
```

`cargo xtask ci ci-local` is the project's CI. It runs locally only, because the reference trees it
checks are not in the repository. `cargo xtask dist` never copies anything into a mod folder or into
TeamSpeak; installing is a manual step
([manual install and test](/documentation/runbooks/manual_install_and_test.md)).

## Status

All code is written and released as `v0.1.1`. The licence is not decided yet; the upstream mods are
under APL-SA.

## Open work

- [ ] Live check: both plugins load in the Flatpak TeamSpeak (runbook, TeamSpeak section).
- [ ] Live check: the probe answers `PONG` under Proton's wine, then `TS_INFO\tPING` answers in Arma
      with BattlEye off.
- [ ] Live check: install from `target/dist/` by hand and play both mods; listen for clicks on
      fast-moving sources (does the HRTF need its own crossfade?) and in other players' voices.
- [ ] Make `split_keeping_every_field` in `tfar_arma_values` public and use it in the 1.0 game command
      parser instead of its private copy.
- [ ] Document `append_own_voice_for_speakers` (1.0 voice pipeline) as off-audio-thread only, or give
      it a `StreamProducer` parameter, as the 0.9 `append_own_voice`.
