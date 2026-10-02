**Status:** frozen record (2026-10-02)

# Library survey

Each need was checked against crates.io on 2026-10-02, and the packaged source was read for ts3plugin,
ts3plugin-sys, arma-rs, biquad, sci-rs, iir_filters, fundsp, idsp, hrtf, rustdct and eframe. The chosen set
is decision D18–D22.

## Chosen
| Need | Crate | Version (date) | Licence | Notes |
|---|---|---|---|---|
| TS3 bindings | `ts3plugin-sys` | 0.5.0 (2024-10-03) | MIT/Apache-2.0 | API 26 (TS 3.6). Raw `#[repr(C)] Ts3Functions` with every function we need; no export types. Low activity |
| Win32 | `windows-sys` | 0.61.2 | MIT/Apache-2.0 | Features `Win32_UI_WindowsAndMessaging`, `Win32_System_Environment`, `Win32_Foundation`; `windows-link` may need mingw `dlltool` |
| Vector maths | `glam` | 0.33.12 (2026-10-01) | MIT/Apache-2.0 | f32/f64 vectors |
| Butterworth design | `sci-rs` | 0.4.1 (2024-11-22, repo active 2026-05) | MIT/Apache-2.0 | `butter_dyn(N, [lo, hi], Bandpass, fs, Sos)` scipy port; heavy deps (nalgebra, ndarray); design time only |
| RBJ + DF2T | `biquad` | 0.6.0 (2026-03-22) | MIT/Apache-2.0 | RBJ LP/HP with Q; DF1 and DF2T processing |
| HRTF | `hrtf` | 0.9.0 (2026-08-06) | MIT | Needs `.bin` HRIR spheres from mrDIMAS/hrir_sphere_builder (IRCAM, 44.1 kHz, resampled at load via rubato). Overlap-save; input = `interpolation_steps · block_len`; mono in, adds into interleaved stereo; per-source state; clicks on fast-moving sources |
| FFT | `rustfft` / `realfft` | 6.4.1 / 3.5.0 | MIT/Apache-2.0, MIT | Only if needed beyond `hrtf` |
| WAV | `hound` | 3.5.1 | Apache-2.0 | — |
| SQLite | `rusqlite` (`bundled`) | 0.40.2 | MIT | `cc`-built SQLite |
| HTTP | `ureq` | 3.4.2 (2026-09-13) | MIT/Apache-2.0 | Blocking, rustls |
| Framing | `postcard` + `serde` | 1.1.3 + 1.0.229 | MIT/Apache-2.0 | — |
| Concurrency | `parking_lot`, `crossbeam-channel`, `arc-swap`, `rtrb` | 0.12.5, 0.5.17, 1.9.2, 0.4.0 | MIT/Apache-2.0 | `rtrb` for real-time-safe audio queues |
| Logging | `tracing`, `tracing-subscriber`, `log` | 0.1.44, 0.3.23, 0.4.34 | MIT | A layer forwards to TS `logMessage` from a non-audio thread |
| Profiling | `profiling`, `tracy-client`, `tracing-tracy` | 1.0.18, 0.19.0, 0.12.0 | MIT/Apache-2.0 | Tracy builds C++ with `cc`; feature-gated |
| Debug window | `eframe`/`egui` | 0.36.2 (2026-09-08) | MIT/Apache-2.0 | `default-features = false, features = ["glow", "x11", "default_fonts"]`; winit 0.30 X11 `with_any_thread(true)`; one event loop per process, so keep one long-lived UI thread and join it at shutdown. Flatpak TS has `--socket=x11 --device=dri` |
| Errors | `thiserror`, `anyhow` | 2.0.21, 1.0.104 | MIT/Apache-2.0 | `anyhow` in tools only |
| Mock host | `libloading` | 0.9.0 | ISC | — |
| xtask | `clap`, `xshell`, `cargo_metadata`, `toml_edit`, `sha2`, `walkdir`, `ignore` | 4.6.7, 0.2.7, 0.23.1, 0.25.15, 0.11.0, 2.5.0, 0.4.33 | permissive | — |

## Rejected
| Crate | Why |
|---|---|
| `ts3plugin` 0.3.1 (high level) | Missing onCustom3dRolloffCalculationClientEvent, onClientSelfVariableUpdateEvent, infoData/freeMemory, menus, hotkeys; every callback locks one global `std::Mutex` with `.unwrap()` |
| `arma-rs` 1.13.0 | Looks the whole plain string up as a command name: `"TS_INFO\tPING"` fails and `:` triggers group lookup; no raw handler. Licence metadata conflicts (MIT vs repo GPL-2.0) |
| `bincode` 3.0.0 | Placeholder that refuses to compile; 2.x unmaintained |
| `fundsp` 0.23, `idsp` 0.23, `dasp` 0.11, `iir_filters` 0.1.3, `synfx-dsp` (GPL-3.0) | No Butterworth band-pass with centre + width, stale, or licence |
| `compressor` 0.3, `audio-processor-dynamics` 2.5 | Abandoned / framework-bound |
| `kira`, `oddio`, `rodio`, `fyrox-sound` | Simple pan only, tied to their mixers; no X3DAudio-style calculator |
| `audionimbus` 0.16 | C++ Steam Audio (`libphonon`) — violates pure Rust |
| `slint` | GPL-3.0 / commercial licensing friction |

## Written by us (estimates)
| Part | LoC |
|---|---|
| TS export layer (all `ts3plugin_*`, C strings, thread separation) | 900–1200 |
| Arma exports (`RVExtension`, `RVExtensionVersion`, truncating writes) | 150 |
| Localhost link (framing, reconnect, timeouts) | 350 |
| Filter wrappers + multi-channel state | 200 |
| Compressor | 120 |
| Distance curve + cone gain | 150 |
| Sample helpers | 30 |
| tracing → TS log layer | 80 |
| Mock TS host | 500 |
| Debug window panels | 400 |
| xtask | 600 |
