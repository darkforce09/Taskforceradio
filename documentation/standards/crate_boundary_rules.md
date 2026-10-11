# Crate boundary rules

Where a crate sits, what it may depend on, which external crates it may use and where `unsafe` is
allowed. `tools/xtask/src/layout.rs` holds these rules as data; `cargo xtask verify crate-tiers`,
`dependency-firewalls` and `unsafe-allowlist` enforce them, and this page and that module change
together.

## Layout metadata and tiers

Every member declares its place in its manifest:

```toml
[package.metadata.layout]
category = "crates/audio"   # crates/<category> or tools
tier = 1                    # 0 with no workspace dependency, else 1 + the highest dependency tier
targets = "any"             # "windows" for a crate built only for x86_64-pc-windows-gnu
```

The crate's path is its category plus its name, and the folder name equals the package name. Normal
and build dependencies point strictly down the tiers; dev-dependencies are exempt from the tier order.

The four deliverables (`tfar_plugin_0_9`, `tfar_extension_0_9`, `tfar_plugin_1_0`,
`tfar_extension_1_0`) sit in their version category and are known to the gates by name. Plugins are
`cdylib` Linux `.so` files; extensions are `cdylib` Windows DLLs. Each is also an `rlib` so tests and
the mock host can link it.

## Category edge matrix

A deliverable's row replaces its category's row for that crate.

| From | May depend on |
|---|---|
| crates/foundation | crates/foundation |
| crates/audio | crates/foundation, crates/audio |
| crates/teamspeak | crates/foundation, crates/teamspeak |
| crates/game_link | crates/foundation, crates/game_link |
| crates/services | crates/foundation |
| crates/interface | crates/foundation |
| crates/version_0_9 libraries | crates/foundation, crates/audio, crates/teamspeak, `tfar_game_link_protocol`, crates/services, crates/interface, crates/version_0_9 |
| crates/version_1_0 libraries | crates/foundation, crates/audio, crates/teamspeak, `tfar_game_link_protocol`, crates/services, crates/interface, crates/version_1_0 |
| `tfar_plugin_0_9`, `tfar_plugin_1_0` | every crates/* category except the other version's |
| `tfar_extension_0_9`, `tfar_extension_1_0` | crates/foundation, crates/game_link |
| tools | every crates/* category |

- **version_0_9 and version_1_0 never depend on each other**, in any kind of dependency.
- No crate, tool or dev-dependency depends on a deliverable.

## External-crate firewalls

An external crate appears as a direct dependency only in the crates listed. Crates not listed
(`thiserror`, `tracing`, `parking_lot`, `crossbeam-channel`, `arc-swap`, `rtrb`, `glam`, `serde`,
`profiling`, `version-compare`) are free for libraries.

| External crate | Only in | Why |
|---|---|---|
| `windows-sys` | `tfar_arma_extension_abi` | Win32 calls stay in the Windows FFI crate |
| `ts3plugin-sys`, `libc` | `tfar_ts3_host`, `tfar_ts3_mock` | the TeamSpeak C ABI and its allocator |
| `rusqlite` | `tfar_ts3_settings_db` | bundled SQLite stays out of the FFI crate |
| `ureq`, `url` | `tfar_web_requests` | the TLS stack stays out of the audio path |
| `eframe`, `egui`, `winit` | `tfar_debug_window` | the GUI stack |
| `sci-rs`, `biquad` | `tfar_dsp_filters` | filter design |
| `hrtf`, `rustfft`, `realfft` | `tfar_hrtf` | the convolution engine |
| `hound` | `tfar_playback_mixer` | WAV decoding |
| `fastrand` | `tfar_radio_effects_0_9`, `tfar_radio_effects_1_0` | radio noise |
| `postcard` | the game_link crates | the wire encoding |
| `libloading` | `tfar_ts3_mock`, `tfar_extension_probe` | loading built plugins and DLLs |
| `tracy-client` | the two plugins, behind the `tracy` feature | profiling |
| `cc` | `tfar_golden_vectors` | compiling the C++ oracles |
| `anyhow` | the tools | libraries and deliverables use `thiserror` |
| `clap`, `xshell`, `cargo_metadata`, `toml_edit`, `sha2`, `walkdir`, `ignore` | `xtask` | the task runner |

## Unsafe allow-list

`unsafe_code` is denied workspace-wide. Only these crates lift it, with `#![allow(unsafe_code)]` at the
crate root and a reason comment on the line above:

- `tfar_ts3_host` — the TeamSpeak C ABI;
- `tfar_arma_extension_abi` — the Arma C ABI and Win32 calls;
- the four deliverables — their export glue only;
- `tfar_ts3_mock` — loads plugins and fakes TeamSpeak's function table;
- `tfar_extension_probe` — loads the extension DLL and calls `RVExtension`;
- `tfar_golden_vectors` — calls the compiled C and C++ oracles.

## Features

- `test_fixtures` is the only feature a library declares, enabled only from `[dev-dependencies]`.
- `tracy` and `api_profiler` exist only in the two plugins and forward to `profiling`'s backends.
