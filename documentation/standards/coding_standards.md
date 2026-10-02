**Status:** live

# Coding standards

The rules for the Rust code of the two TeamSpeak plugins, the two Arma extensions, the shared libraries
and the tools. A rule held by a program names the `cargo xtask verify` gate, the lint or the test that
holds it; the rest are held in review.

## Language

- Rust only. Every tracked file outside `reference/` is Rust, Markdown, TOML or JSON, apart from
  the Cargo lock file, the git and editor configuration files and binary assets (WAV, HRIR `.bin`). No shell,
  Python, Node or Makefile, for any purpose (`cargo xtask verify language-bans`).
- C and C++ appear only in tools/tfar_golden_vectors: its oracle/ folder (the vendored FAudio
  `F3DAudio.c`) and its shim/ folder, compiled with `cc` from a build script, plus the original DSPFilters,
  SimpleComp and clunk sources compiled straight from `reference/` (decision D9). Product code never
  links them. The one other place a C file may sit is `crates/teamspeak/tfar_ts3_host/sdk_headers/`,
  and only as `.h`: the TeamSpeak SDK headers, read as text by the layout tests and never compiled.
- Repository chores are `cargo xtask` commands; a one-off investigation is a test or an xtask command,
  never a script.

## Workspace and manifests

- One workspace, `resolver = "3"`, with one member glob each for the apps, the crates by category and the
  tools (`"apps/*"`, `"crates/*/*"`, `"tools/*"` in the root manifest).
- `[workspace.package]` sets edition 2024, `rust-version = "1.99"`, `license = "UNLICENSED"` and
  `publish = false`; every crate inherits them with `*.workspace = true`.
- Every manifest opens with a one- to three-line comment saying what the crate is and who uses it, and
  declares its layout:

  ```toml
  [package.metadata.layout]
  category = "crates/audio"
  tier = 1
  targets = "any"
  ```

  `targets` is `"any"`, or `"windows"` for a crate built only for `x86_64-pc-windows-gnu`.
- Every dependency is `name.workspace = true`, with a comment above it saying why the crate needs it.
  `[lints] workspace = true` in every crate (`cargo xtask verify crate-anatomy`).

## Dependency policy

- **Use libraries (D18).** A need with a fitting crate on the library map uses that crate; we write only
  the glue the map lists. A new external crate needs a line in the library map of the
  [workspace decisions](/documentation/decisions/workspace.md) first.
- **Workspace pins.** Every external crate is pinned once in `[workspace.dependencies]`, with
  `default-features = false` and only the features in use turned on, and a comment saying why. Internal
  crates are listed there by path, under `# crates/<category>:` banners.
- **Firewalls.** An external crate appears only in the crates allowed to use it
  (`cargo xtask verify dependency-firewalls`):

  | External crate | Only in |
  |---|---|
  | `windows-sys` | `tfar_arma_extension_abi` |
  | `ts3plugin-sys` | `tfar_ts3_host`, `tfar_ts3_mock` |
  | `libc` | `tfar_ts3_host`, `tfar_ts3_mock` |
  | `rusqlite` | `tfar_ts3_settings_db` |
  | `ureq`, `url` | `tfar_web_requests` |
  | `egui`, `eframe`, `winit` | `tfar_debug_window` |
  | `sci-rs`, `biquad` | `tfar_dsp_filters` |
  | `hrtf`, `rustfft`, `realfft` | `tfar_hrtf` |
  | `hound` | `tfar_playback_mixer` |
  | `fastrand` | `tfar_radio_effects_0_9`, `tfar_radio_effects_1_0` |
  | `postcard` | the game_link crates |
  | `libloading` | `tfar_ts3_mock`, `tfar_extension_probe` |
  | `tracy-client` | the two plugin apps, behind the `tracy` feature |
  | `cc` | `tfar_golden_vectors` |
  | `anyhow` | the tools |
  | `clap`, `xshell`, `cargo_metadata`, `toml_edit`, `sha2`, `walkdir`, `ignore` | xtask |

  The general-purpose crates (`thiserror`, `tracing`, `parking_lot`, `crossbeam-channel`, `arc-swap`,
  `rtrb`, `glam`, `serde`, `profiling`, `version-compare`) are free for libraries. The authoritative
  table is in [laws and gates](/documentation/program/laws_and_gates.md#external-crate-firewalls).

- **Supply chain.** `cargo deny check` holds licences (an allow-list), sources (crates.io only; no git
  dependency) and advisories; every ignore carries a reason. One version per crate in the tree is the aim;
  `multiple-versions` turns from warn to deny once the real dependencies land.
- **Features.** `test_fixtures` may exist in any crate; `tracy` and `api_profiler` only in the plugin apps,
  where they forward to `profiling` (`cargo xtask verify crate-anatomy`).

## Layering

- Tier 0 is a crate with no workspace dependency; any other crate's tier is 1 plus the highest tier it
  depends on, so edges point strictly down (`cargo xtask verify crate-tiers`).
- The category edge matrix: foundation depends on nothing; audio, teamspeak and game_link on foundation
  and their own category; services and interface on foundation; version_0_9 and version_1_0 on
  foundation, audio, teamspeak, `tfar_game_link_protocol`, services and interface. **version_0_9 and
  version_1_0 never depend on each other.** Plugin apps use anything except the other version's crates;
  extension apps use only foundation and game_link; tools use crates, never apps. Dev-dependencies are
  exempt from tier order but never point at apps (`cargo xtask verify crate-tiers`).
- A folder name equals its package name, and the path equals category plus name.

## Crate anatomy

- `src/lib.rs` stays at or under 80 lines and holds only the `//!` header, crate attributes, `mod` lines,
  `pub mod` and `pub use`.
- `src/prelude.rs` re-exports what a user imports; a `pub use` of another workspace crate appears only in
  `prelude.rs`.
- `src/error.rs` holds the crate's `thiserror` `Error` enum and `pub type Result<T>`.
- `README.md` (crate root or app kind) and `src/README.md` (domain or leaf kind).
- Apps are `crate-type = ["cdylib", "rlib"]` with `[lib] name` equal to the crate name; `cargo xtask dist`
  gives the artifacts their final names.

All of it is checked by `cargo xtask verify crate-anatomy`; `cargo xtask new-crate` writes it.

## Errors

- Libraries and apps return typed errors made with `thiserror`; `anyhow` is for tools only
  (`cargo xtask verify crate-anatomy`).
- An error carries what a reader needs to act: the token, the frame kind, the path, the TeamSpeak error
  code.
- At a host boundary (a `ts3plugin_*` export, `RVExtension`) an error becomes the host's answer: a log line
  through TeamSpeak's `logMessage`, a message in the current tab, or the game-facing reply string the
  original sends. It never becomes a panic.

## Panic-free code

- `panic = "abort"` in the dev and release profiles (decision D12); a panic in a plugin would take
  TeamSpeak down, and in an extension, Arma. The test harness still unwinds.
- Denied in production code: `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo`,
  `unimplemented`, `dbg_macro` (`cargo clippy --workspace --all-targets -- -D warnings`). The root
  clippy.toml allows unwrap, expect, indexing and panic in tests.
- Use `get`, `split_first`, iterators and pattern matching instead of indexing; `checked_*` or
  `saturating_*` arithmetic where an overflow is possible. Sample arithmetic saturates: a gain never wraps.
- Every token of a game message is read by position with a length check; a short message is an error
  value, never an out-of-bounds read.
- An allowed lint carries a `reason = "…"`.

## Unsafe code and FFI

- `unsafe_code` is denied workspace-wide. Only `tfar_ts3_host`, `tfar_arma_extension_abi`, the four apps
  (their export glue), `tfar_ts3_mock`, `tfar_extension_probe` and `tfar_golden_vectors` may state `#![allow(unsafe_code)]` at the crate root, with a
  reason comment on the line above (`cargo xtask verify unsafe-allowlist`). Everything else uses their safe wrappers.
- Every `unsafe` block has a `// SAFETY:` comment directly above it saying which invariant makes it sound
  (`undocumented_unsafe_blocks`, `cargo xtask verify unsafe-allowlist`); every `unsafe fn` has a
  `# Safety` section; `unsafe_op_in_unsafe_fn` is denied.
- **Never unwind across FFI.** Exports are `extern "C"` and the profiles abort on panic; an export's body
  turns every error into a return value, so nothing reaches the boundary but a value.
- **C strings in.** A `*const c_char` from TeamSpeak or Arma is checked for null before `CStr::from_ptr`
  and read as bytes; text is decoded explicitly (lossily where the original accepts any bytes). Its
  lifetime ends with the call: nothing keeps the pointer.
- **C strings out.** A write into a host buffer (`RVExtension`'s `output`, TeamSpeak's return buffers)
  writes at most size − 1 bytes and a NUL, truncating at a character boundary. Memory handed to TeamSpeak
  that TeamSpeak frees through `ts3plugin_freeMemory` is allocated by the allocator that function frees
  with; memory TeamSpeak hands over is freed with its `freeMemory` function. Each `// SAFETY:` comment
  names who owns the memory.
- **Audio buffers.** A sample pointer and its frame and channel counts become a slice only after a null
  check and a size check against the counts.
- **Layout.** Every `#[repr(C)]` struct shared with a host has `size_of` and `offset_of` tests against the
  published SDK headers.
- **Global state.** Plugin and extension state lives in a `static` with interior mutability (`OnceLock`,
  `ArcSwap`, a mutex); never `static mut`.

## Threads and the TeamSpeak audio callbacks

TeamSpeak calls the voice callbacks (`onEditPlaybackVoiceDataEvent`, `onEditPostProcessVoiceDataEvent`,
`onEditMixedPlaybackVoiceDataEvent`, `onEditCapturedVoiceDataEvent`) on its audio thread, frame by frame.
A late frame is an audible glitch, and a blocked one stalls TeamSpeak's audio.

- **Read state through snapshots.** The game link thread publishes the game state as an immutable
  snapshot in an `arc-swap` `ArcSwap`; the audio thread loads it without locking.
- **No blocking lock held across a TeamSpeak call.** No thread calls a TeamSpeak function while holding a
  lock the audio thread or a TeamSpeak callback can take; TeamSpeak may call back on the calling thread.
- **The audio thread never waits.** Where it must touch shared mutable state, it uses `try_lock` and, on
  contention, passes the frame through as the original does when data is missing.
- **Hand-offs by queue.** Samples and commands between the audio thread and others travel through `rtrb`
  ring buffers or `crossbeam-channel` channels sized up front.
- **Allocation.** Avoid allocation-heavy work on the audio thread: filters, effects and scratch buffers
  are created once per client or radio and reused; a `Vec` grows only on the first frames of a source.
- **Logging.** The audio thread logs only at trace level and never per frame at higher levels.
- **Thread ownership.** Every module's `//!` header names the threads that call it under Signals and
  state. The plugin's long-running threads (game link listener, service tick, async command queue, debug
  window) are named and stopped in `ts3plugin_shutdown` before it returns.

## Naming

- Crates: `tfar_` prefix, snake_case, folder name equal to package name, version tag `_0_9` or `_1_0` on
  every version crate.
- Names need zero context: no abbreviation a reader must look up, except the domain terms in the
  [glossary](/documentation/glossary.md) (SW, LR, DD, PTT, VAD, HRIR, RBJ).
- Types are nouns, functions verbs; a boolean reads as a question (`is_dormant`, `can_use_long_range`).
- An identifier of a client, server, radio or antenna is a newtype, never a bare integer in a public API.
- Original C++ names are kept only where they are wire format: command names, reply strings, metadata
  tags.

## Module layout

- One concept per module; related variants in a named subfolder (the radio effects, the frame kinds, the
  gate commands), never a flat dump.
- A module folder has `mod.rs`, or the file of the same name beside it, holding only `mod` lines and
  re-exports.
- Version crates keep the behaviour of one mod; what both mods share lives below them, in foundation,
  audio, teamspeak or game_link.

## Comments

- Crate roots and every production file of 80 lines or more open with the four-point `//!` header:
  **Role:**, **Position:**, **Signals and state:**, **Invariants:**
  (`cargo xtask verify module-headers`; [template](/documentation/standards/templates/module_header.md)).
- Every public item has `///` docs with intra-doc links (`missing_docs`).
- Present tense; no history, no milestone or finding ids, no "ported from" or "fixed" (review). The
  `reference/` path of a ported behaviour may be named as the specification it follows.
- Comment the why: the invariant, the unit, the host constraint, the reason a value is what it is.

## Tests

- Unit tests live in sibling files: `#[cfg(test)] #[path = "tests/<module>.rs"] mod tests;`; no inline
  test module and no test function in a production file (`cargo xtask verify test-placement`).
- Test names are sentences of what holds (`a_short_pos_message_is_an_error`).
- Integration tests live in the crate's `tests/` folder; host scenarios run against a built plugin
  through `tfar_ts3_mock`.
- Audio crates compare against golden vectors from `tfar_golden_vectors` within a stated tolerance.
- Every fixed original bug has a test named in the program's bug fixes record.
- Tests are never deleted, ignored or loosened to get green; every new check gets a perturbation proof:
  break what it guards, show it fails, restore.

## File size and formatting

- Production files at or under 500 lines, test files at or under 1000 (`cargo xtask verify file-length`).
  No allowlist: a file that grows splits by responsibility.
- `cargo fmt` with `style_edition = "2024"`; the gate is `cargo fmt --all --check`.
- UTF-8, LF line ends, a final newline, no trailing whitespace outside Markdown
  (`cargo xtask verify editorconfig`, mirroring the root .editorconfig).

## Logging and profiling

- `tracing` everywhere; the plugins install a layer that forwards to TeamSpeak's `logMessage`.
- Profiling goes through `profiling`, with Tracy behind the plugin apps' `tracy` feature.

## Related documentation

- [Workspace decisions](/documentation/decisions/workspace.md) — the conventions and the library map.
- [Laws and gates](/documentation/program/laws_and_gates.md) — the edge matrix, the firewalls, every gate.
- [Module header template](/documentation/standards/templates/module_header.md) — the four-point header.
- [README standard](/documentation/standards/readme_standard.md) — the README every code folder carries.
