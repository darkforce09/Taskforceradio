# CLAUDE.md — Taskforceradio

Working context for AI sessions. Read this first, then [documentation/README.md](/documentation/README.md).
`AGENTS.md` is a symlink to this file; edit this one.

Taskforceradio is a from-scratch Rust rewrite of the native parts of Task Force Arrowhead Radio (TFAR),
the Arma 3 radio mod, for one setup: Arma 3 as the Windows build under Proton and TeamSpeak 3 as the native
Linux Flatpak. The originals link game and TeamSpeak through Win32 named objects that a native TeamSpeak
plugin cannot reach, so both mods get new parts: per mod, one Arma extension (a Windows x64 DLL) and one
TeamSpeak plugin (a Linux `.so`), talking over localhost TCP. Shared logic lives in library crates. The
mods themselves are archived byte for byte in `reference/` and never edited; no SQF changes, and the
extension keeps the name `task_force_radio_pipe`.

**Current state:** milestone M0 (workspace skeleton, reference archive, gates and documentation) is in
progress. Resume from [progress.md](/documentation/program/progress.md), which holds the current milestone,
the last green commit, the next action and the handoff; the milestones are in the
[roadmap](/documentation/roadmap.md).

## 1. Project laws

Each law names the gate that holds it; "review" means the orchestrator holds it when it reads a slice.
The detail is in the [standards](/documentation/standards/README.md).

1. **No silent deferrals.** Do the whole ask. Only the owner defers work, in so many words ("defer X",
   "skip X"). Plan wording such as "optional" or "later" and agent-written deferral lists are not
   permission; what truly cannot be done is reported with what and why. Gate: review of every agent
   report's "Not done" section; deferrals the owner grants are recorded in progress.md.
2. **Rust only for product code.** Every tracked source file is Rust; the rest is Markdown, TOML or JSON.
   No Python, shell, Makefiles or Node, for any purpose; a repository task is a `cargo xtask` command. C
   and C++ appear only as the golden-vector oracle in tools/tfar_golden_vectors (decision D9). Gate:
   `cargo xtask verify language-bans`.
3. **`reference/` is read-only.** It holds the two original mods byte for byte. Read it freely; never
   write, format, move or touch a file under it, `reference/README.md` excepted. Gate:
   `cargo xtask verify reference-untouched` against the checksum list reference/MANIFEST.sha256.
4. **Use libraries, don't reinvent (D18).** Every need with a fitting crate uses it (the library map in
   the [workspace decisions](/documentation/decisions/workspace.md)); we write only the glue the map
   lists. Gate: review against the library map; `cargo xtask verify dependency-firewalls` and
   `cargo deny check` hold where each library may be used.
5. **Strict category layering.** A crate depends only on crates of a lower tier and of the categories
   the edge matrix allows; `version_0_9` and `version_1_0` never depend on each other; extension apps use
   only foundation and game_link; tools never use apps. Gate: `cargo xtask verify crate-tiers`.
6. **Self-describing names.** Folder, file, module and symbol names need zero context: snake_case, no
   cryptic abbreviations, every crate prefixed `tfar_`, the folder name equal to the package name,
   version tags `_0_9` and `_1_0`. Gate: `cargo xtask verify crate-tiers` (folder = package name);
   the rest in review.
7. **Variants go in subfolders.** No flat dumps: related variants (the radio effects, the frame kinds,
   the gate commands) are grouped into a named subfolder. Gate: review; `file-length` forces the split.
8. **File size ceilings.** Production files stay at or under 500 lines, test files at or under 1000. No
   allowlist and no exemption: a file that grows splits by responsibility. Gate:
   `cargo xtask verify file-length`.
9. **Sibling test files.** Unit tests live in `#[cfg(test)] #[path = "tests/<module>.rs"] mod tests;`;
   no inline test module. Gate: `cargo xtask verify test-placement`.
10. **In-code documentation.** Crate roots and production files of 80 lines or more open with the
    four-point `//!` header (Role, Position, Signals and state, Invariants); every public item has `///`
    docs; comments are present tense, with no history and no milestone or finding ids. Gate:
    `cargo xtask verify module-headers`, the `missing_docs` lint; the wording in review.
11. **No panics in production code.** `panic = "abort"` in dev and release; `unwrap_used`, `expect_used`,
    `panic`, `indexing_slicing`, `todo`, `unimplemented` and `dbg_macro` are denied, so no panic path
    exists outside tests. Gate: `cargo clippy --workspace --all-targets -- -D warnings`.
12. **`unsafe` only in allow-listed FFI crates.** `tfar_ts3_host`, `tfar_arma_extension_abi`, the four
    apps (export glue) and `tfar_ts3_mock` may hold `#![allow(unsafe_code)]`, with a reason; every
    `unsafe` block has a `// SAFETY:` comment right above it. Gate: `cargo xtask verify unsafe-allowlist`
    and the `undocumented_unsafe_blocks` lint.
13. **Docs move with code.** The comments, the README.md of every folder whose contents, surface or
    boundaries change, and the documents whose behaviour changes land in the same commit as the code.
    Gate: `cargo xtask verify readme-coverage`, `readme-sections`, `status-lines`, `markdown-placement`
    and `link-check`.
14. **Git.** Conventional Commits straight to `main`, staged with explicit paths (never `git add -A`),
    each ending with a `Co-Authored-By:` trailer; no remote yet. During a program only the orchestrator
    commits ([commit conventions](/documentation/standards/commit_conventions.md)). Gate: review.

## 2. Directory atlas

The target layout of the approved plan. M0 creates all of it as compiling skeletons; a folder's own
README.md says what it holds today.

```text
Taskforceradio/
├── Cargo.toml  Cargo.lock      the workspace: resolver 3, member globs, workspace package, deps, lints
├── rust-toolchain.toml         Rust 1.99.0 with the Linux and windows-gnu targets
├── rustfmt.toml  clippy.toml  deny.toml  typos.toml  .editorconfig
├── .cargo/config.toml          the xtask alias; windows-gnu linker x86_64-w64-mingw32-gcc, +crt-static
├── .gitignore  .gitattributes
├── README.md                   what the project is, layout, build and check, status
├── CLAUDE.md                   this file;  AGENTS.md -> CLAUDE.md
├── apps/                       the four deliverables (cdylib + rlib)
│   ├── tfar_plugin_0_9/        TS plugin .so for 0.9.12: exports, threads, events, PTT/VAD, metadata,
│   │                           serious mode
│   ├── tfar_plugin_1_0/        TS plugin .so for 1.0: the same plus the async queue, antennas, SETCFG
│   │                           and the focus-aware channel move
│   ├── tfar_extension_0_9/     Arma extension DLL for 0.9.12: every call is a sync round trip
│   └── tfar_extension_1_0/     Arma extension DLL for 1.0: `~` async, DFRAME → NEEDCFG/OK, MISSIONEND,
│                               FOCUS frames
├── crates/                     library crates, crates/<category>/<crate>/
│   ├── foundation/             tfar_geometry (glam domain helpers), tfar_arma_values
│   ├── audio/                  tfar_sample_buffer, tfar_dsp_filters, tfar_dynamics, tfar_hrtf,
│   │                           tfar_spatial_audio (distance curve + cone), tfar_playback_mixer (hound)
│   ├── teamspeak/              tfar_ts3_host (export layer on ts3plugin-sys), tfar_ts3_settings_db
│   ├── game_link/              tfar_game_link_protocol, tfar_game_link_server, tfar_game_link_client,
│   │                           tfar_arma_extension_abi
│   ├── services/               tfar_web_requests (update check, Piwik; ureq + rustls)
│   ├── interface/              tfar_debug_window (egui window, X11)
│   ├── version_0_9/            tfar_game_commands_0_9, tfar_plugin_commands_0_9,
│   │                           tfar_radio_effects_0_9, tfar_client_state_0_9,
│   │                           tfar_radio_routing_0_9, tfar_voice_pipeline_0_9
│   └── version_1_0/            the same six with _1_0, plus tfar_antennas_1_0 and
│                               tfar_plugin_settings_1_0
├── tools/
│   ├── xtask/                  `cargo xtask`: verify, ci, dist, new-crate, reference-manifest
│   ├── tfar_ts3_mock/          dlopens a plugin .so with a fake TS3Functions table, scripted scenarios
│   ├── tfar_extension_probe/   Windows exe: LoadLibrary + RVExtension, run under Proton's wine
│   └── tfar_golden_vectors/    cc-built reference C++ → fixtures for the audio crates (the one C/C++)
├── assets/hrir/                IRCAM HRIR sphere (.bin) + CREDITS.md
├── documentation/              every document; map and authority ladder in its README.md
│   ├── roadmap.md  glossary.md
│   ├── architecture/           workspace layout, game link protocol, audio pipeline
│   ├── decisions/              the dated decision log, one file per subject
│   ├── standards/              coding, documentation, README and commit rules; templates/
│   ├── program/                plan, crate catalogue, laws and gates, bug fixes, progress,
│   │                           behaviour_specs/, research/, agent_briefs/
│   └── runbooks/               manual install and test; continue in Claude Code
└── reference/                  README.md + MANIFEST.sha256; tfar_1_0/ and tfar_0_9_12/, byte-identical
```

That is 38 crates: 4 apps, 30 libraries and 4 tools. The crate list, tiers and edges are in the
[crate catalogue](/documentation/program/crate_catalogue.md); the edge matrix and firewalls are in
[laws and gates](/documentation/program/laws_and_gates.md).

## 3. Environment

- **Game:** Arma 3, the Windows x64 build, runs under **Proton** (Steam app 107410, its own compatdata
  prefix). Extensions load as `task_force_radio_pipe_x64.dll` from the mod root.
- **BattlEye off for testing.** BattlEye blocks extensions it has not whitelisted, so a new DLL runs only
  with BE off or on a server without BE.
- **TeamSpeak:** 3.6.2 as the Flatpak `com.teamspeak.TeamSpeak3` (plugin API 26). Plugins go in
  `~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/`; the sandbox shares the network and IPC
  with the host, so the plugin's 127.0.0.1 listener is reachable from Proton.
- **Toolchain:** Rust 1.99.0, pinned in `rust-toolchain.toml` with `rustfmt`, `clippy` and the targets
  `x86_64-unknown-linux-gnu` (plugins, tools) and `x86_64-pc-windows-gnu` (extensions, probe).
- **Windows linker:** `x86_64-w64-mingw32-gcc`, set in `.cargo/config.toml` with `+crt-static`, so the
  DLL needs no runtime files beside it.
- **Tools:** `cargo deny` 0.20.2 and `typos` 1.50.3 are installed.
- **Git LFS:** `reference/tfar_0_9_12` holds upstream Git LFS pointer stubs, so a fresh clone
  recreates the line that switches LFS off for reference/ in .git/info/attributes before checkout, or
  clones with `GIT_LFS_SKIP_SMUDGE=1` ([fresh clone](/reference/README.md#fresh-clone)).
- **Paths:** the repository is `/run/media/system/Disk_2/Projects/Taskforceradio`. Nothing builds or
  copies into the game or TeamSpeak folders; `cargo xtask dist` writes only under target/ and the
  owner installs by hand.

## 4. Commands

All run from the repository root, one per shell call. The milestone close runs every one; a slice runs
fmt, clippy, the tests of the crates it touched and the documentation gates.

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p tfar_extension_1_0 -p tfar_extension_0_9 --target x86_64-pc-windows-gnu -- -D warnings
cargo test --workspace
cargo deny check
typos
cargo xtask ci ci-local            # all of the above, then every verify gate, in order
```

The xtask commands (tools/xtask is written in M0):

```bash
cargo xtask verify <gate>          # one gate; exit 0 held, 1 violation, 2 could not run
cargo xtask verify all             # every gate
cargo xtask reference-manifest     # writes reference/MANIFEST.sha256; refuses to overwrite without --force
cargo xtask new-crate <category> <name>   # scaffolds a crate with the standard anatomy
cargo xtask dist                   # release builds into target/dist/tfar_1_0/ and target/dist/tfar_0_9/
```

The gates are `crate-tiers`, `dependency-firewalls`, `crate-anatomy`, `unsafe-allowlist`, `file-length`,
`test-placement`, `module-headers`, `readme-coverage`, `readme-sections`, `status-lines`,
`markdown-placement`, `link-check`, `editorconfig`, `language-bans` and `reference-untouched`; what each
checks is in [laws and gates](/documentation/program/laws_and_gates.md).

## 5. Where to look

| Question | Document |
|---|---|
| Where is the work now, and what is next? | [Progress](/documentation/program/progress.md) |
| What are the milestones and their acceptance tests? | [Roadmap](/documentation/roadmap.md) |
| What is the whole program and how does it run? | [Program plan](/documentation/program/program_plan.md) |
| What is decided, and why? | [Decisions](/documentation/decisions/README.md) |
| Which crate ports which original file? | [Crate catalogue](/documentation/program/crate_catalogue.md) |
| Which edges, firewalls and gates hold? | [Laws and gates](/documentation/program/laws_and_gates.md) |
| Which original bugs are fixed, and where? | [Bug fixes](/documentation/program/bug_fixes.md) |
| What do the original plugins and extensions do? | [Behaviour specs](/documentation/program/behaviour_specs/README.md) |
| How do extension and plugin talk? | [Game link protocol](/documentation/architecture/game_link_protocol.md) |
| How is voice processed? | [Audio pipeline](/documentation/architecture/audio_pipeline.md) |
| How is the workspace laid out? | [Workspace layout](/documentation/architecture/workspace_layout.md) |
| How is code, a README or a commit written? | [Standards](/documentation/standards/README.md) |
| What does a term mean? | [Glossary](/documentation/glossary.md) |
| How do I install and test by hand? | [Manual install and test](/documentation/runbooks/manual_install_and_test.md) |
| How do I resume in a new session? | [Continue in Claude Code](/documentation/runbooks/continue_in_claude_code.md) |
| What is in `reference/`, and how is it restored? | [Reference README](/reference/README.md) |
