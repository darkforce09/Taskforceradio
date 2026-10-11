# Code and documentation

How the Rust code, its comments and the documents are written. Where a checker holds a rule, it is
named; `cargo xtask ci ci-local` runs every checker.

## Language

- Product code is Rust. Tracked files are Rust, Markdown and TOML, plus `Cargo.lock`, the git and
  editor configuration and binary assets (the HRIR sphere). No shell, Python, Node or Makefile for any
  purpose; a repository chore is a `cargo xtask` command (`cargo xtask verify language-bans`).
- C and C++ appear only in `tools/tfar_golden_vectors` (the vendored FAudio oracle and its shim,
  compiled with `cc`, plus original sources compiled straight from `reference/`) and as the TeamSpeak
  SDK headers in `crates/teamspeak/tfar_ts3_host/sdk_headers/`, read as text by layout tests and never
  compiled. Product code never links C.

## Manifests and dependencies

- One workspace, `resolver = "3"`, member globs `crates/*/*` and `tools/*`. Every crate inherits
  edition, `rust-version`, licence and `publish` from `[workspace.package]` and sets
  `[lints] workspace = true`.
- Every external crate is pinned once in `[workspace.dependencies]` with `default-features = false`,
  only the features in use, and a comment saying why; members use `name.workspace = true`.
- Use libraries, don't reinvent: a need with a fitting crate on the library map in the
  [workspace decisions](/documentation/decisions/workspace.md) uses that crate. A new external crate
  gets a line in that map and, if it is heavy or platform-bound, a firewall row in the
  [crate boundary rules](/documentation/standards/crate_boundary_rules.md).
- `cargo deny check` holds licences (allow-list), sources (crates.io only) and advisories; every ignore
  carries a reason.

## Errors and panics

- Libraries and deliverables return typed `thiserror` errors; `anyhow` is for tools only. An error
  carries what a reader needs to act (the token, the frame kind, the path, the TeamSpeak error code).
- At a host boundary (a `ts3plugin_*` export, `RVExtension`) an error becomes the host's answer: a log
  line through TeamSpeak's `logMessage`, a chat message, or the reply string the original sends.
- `panic = "abort"` in the dev and release profiles: a panic in a plugin takes TeamSpeak down, in an
  extension Arma. `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo`, `unimplemented`
  and `dbg_macro` are denied in production code; `clippy.toml` allows them in tests.
- Use `get`, `split_first`, iterators and pattern matching instead of indexing; `checked_*` or
  `saturating_*` arithmetic where overflow is possible. Every token of a game message is read by
  position with a length check. An allowed lint carries a `reason = "…"`.

## Unsafe code and FFI

- `unsafe_code` is denied workspace-wide; only the crates in the
  [unsafe allow-list](/documentation/standards/crate_boundary_rules.md#unsafe-allow-list) lift it
  (`cargo xtask verify unsafe-allowlist`). Every `unsafe` block has a `// SAFETY:` comment directly
  above it naming the invariant and who owns the memory; `unsafe_op_in_unsafe_fn` is denied.
- Never unwind across FFI: exports are `extern "C"` and turn every error into a return value.
- C strings in are null-checked before `CStr::from_ptr` and never kept past the call. Writes into a
  host buffer write at most size − 1 bytes and a NUL, truncating at a character boundary. Memory
  TeamSpeak frees through `ts3plugin_freeMemory` comes from the allocator that function frees with.
- A sample pointer becomes a slice only after a null check and a size check against its counts.
- Every `#[repr(C)]` struct shared with a host has `size_of` and `offset_of` tests against the SDK
  headers. Global state lives in a `static` with interior mutability, never `static mut`.

## Threads and the TeamSpeak audio callbacks

TeamSpeak calls the voice callbacks on its audio thread, frame by frame; a late frame is an audible
glitch and a blocked one stalls TeamSpeak's audio.

- The game link thread publishes game state as an immutable snapshot in an `ArcSwap`; the audio thread
  loads it without locking.
- No thread calls a TeamSpeak function while holding a lock the audio thread or a TeamSpeak callback can
  take.
- A voice callback may wait only for a lock whose holders never call TeamSpeak and keep their work
  short ([decision](/documentation/decisions/audio.md)); every other lock on the audio thread is taken
  with `try_lock`, skipping that source for the frame.
- Samples and commands cross threads through `rtrb` ring buffers or `crossbeam-channel` channels sized
  up front. Filters, effects and scratch buffers are created once per client or radio and reused.
- The audio thread logs only at trace level. The plugin's long-running threads (game link listener,
  service tick, async command queue, debug window) are named and stopped in `ts3plugin_shutdown`.

## Names and layout

- Crates carry the `tfar_` prefix, snake_case, folder name equal to package name, and the version tag
  `_0_9` or `_1_0` on every version crate.
- Names need no context: no abbreviation a reader must look up, except the domain terms in the
  [glossary](/documentation/glossary.md) (SW, LR, DD, PTT, VAD, HRIR, RBJ). Types are nouns, functions
  verbs, booleans questions (`is_dormant`). Client, server, radio and antenna identifiers are newtypes.
- Original C++ names survive only where they are wire format: command names, reply strings, metadata
  tags.
- One concept per module; related variants go in a named subfolder (the radio effects, the frame kinds),
  never a flat dump. Production files stay at or under 500 lines, test files at or under 1,000
  (`cargo xtask verify file-length`).
- `cargo fmt` with `style_edition = "2024"`; UTF-8, LF, final newline, no trailing whitespace outside
  Markdown (`cargo xtask verify editorconfig`).

## Comments

- Comments say what the code does and why: the invariant, the unit, the host constraint, the reason a
  value is what it is. Present tense; no history, ticket, milestone, finding or decision ids, no
  "ported from" or "fixed" (`cargo xtask verify history-residue`).
- A behaviour of the original mods is cited by `reference/` path and line
  (`reference/tfar_1_0/ts/src/plugin.cpp:812`) as the specification the code follows.
- `///` docs on every public item (`missing_docs`); a `//!` line where a module's purpose is not
  obvious from its name, naming the threads that call it where that matters.

## Documents

- One README per crate, per `crates/<category>`, per top-level folder and per documentation area: what
  it is for, how to use it, its boundaries. No file-tree listings, no status lines.
- Documents live under `documentation/`; a feature document exists only for user-visible behaviour.
  Open work lives in the checklist in the root [README](/README.md), never in documents.
- Links start at the repository root (`/documentation/glossary.md`); every link, anchor and backticked
  repository path resolves (`cargo xtask verify link-check`).
- A decision entry in `documentation/decisions/` is `### YYYY-MM-DD — <the decision>` with Context,
  Decision and Consequences; a changed decision gets a new entry naming the one it supersedes.
- When a change alters a surface, command or boundary, its README and documents change in the same
  commit.

## Commits

Conventional Commits straight to `main`: `type(scope): lowercase summary`, at most 72 characters; the
scope is the crate in kebab-case, `xtask`, `workspace` or `docs`. The body says what changed and why.
Never commit build output or machine-local settings.
