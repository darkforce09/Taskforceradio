**Status:** live

# Workspace decisions

What is built, in which language, where it lives, how it is named and laid out, how it is committed,
built and shipped, and which libraries it uses. The ids D1 to D22 are the approved program plan's.

### 2026-10-02 — Everything native is rewritten in Rust (D1)

**Context:** Both TFAR versions link game and TeamSpeak through Win32 named objects (0.9.12 the named
pipe `\\.\pipe\task_force_radio_pipe`, 1.0 the shared memory `Local\TFARSHAMEM` with events and a mutex).
Under Proton these live inside wineserver, where a native TeamSpeak plugin cannot reach them, and the
installed plugins are Windows DLLs. Porting the C++ piece by piece would keep its Win32 coupling and its
bugs.

**Decision:** Rewrite everything native in Rust, from scratch: both Arma extensions, both TeamSpeak
plugins and all of the DSP. One extension and one plugin per mod: tfar_extension_1_0 with tfar_plugin_1_0,
tfar_extension_0_9 with tfar_plugin_0_9. Shared logic goes in library crates. The SQF and the extension
name `task_force_radio_pipe` stay as they are.

**Consequences:** The originals become the specification, read from `reference/` and recorded in the
behaviour specs. No C++ is linked into a product (decision D9 allows it only as a test oracle).

**Supersedes:** none.

### 2026-10-02 — The repository root holds the port, the mods sit read-only in reference/ (D5)

**Context:** The two mod trees sat beside each other in the project folder with nothing around them.

**Decision:** `Taskforceradio/` is the repository root. The mods move, unchanged, to `reference/tfar_1_0/`
and `reference/tfar_0_9_12/`, and are read-only from then on.

**Consequences:** No file under `reference/` is ever written, formatted or moved; `reference/README.md` is
the one exception. `cargo xtask verify reference-untouched` holds this against the manifest (see D15).

**Supersedes:** none.

### 2026-10-02 — Crate names carry the tfar_ prefix and a version tag (D6)

**Context:** 38 crates share one workspace and two of them per role differ only by mod version.

**Decision:** Every crate is prefixed `tfar_`, in snake_case, and its folder name equals its package name.
Version crates end in `_1_0` or `_0_9`.

**Consequences:** `cargo xtask verify crate-tiers` checks folder = package name and path = category +
name. Commit scopes are the package names in kebab-case.

**Supersedes:** none.

### 2026-10-02 — Deliverables in apps, libraries in crates by category, tooling in tools (D7)

**Context:** The workspace needs one obvious place for each kind of crate.

**Decision:** apps/ holds the four deliverables, crates/<category>/<crate>/ the libraries, and
tools/ the tooling. The categories are foundation, audio, teamspeak, game_link, services, interface,
version_0_9 and version_1_0.

**Consequences:** Every manifest declares `[package.metadata.layout]` with its category, tier and targets;
the category edge matrix and the version ban are checked by `cargo xtask verify crate-tiers`.

**Supersedes:** none.

### 2026-10-02 — Conventions follow TBD-Reforger and rerun, milestones TBD-subtitles (D8)

**Context:** The owner's other Rust workspaces already have tested conventions and gates.

**Decision:** Conventions follow TBD-Reforger and rerun: workspace package, dependencies and lints, tier
metadata, the crate anatomy, `cargo xtask verify` gates, cargo-deny and typos. Milestones live in
`roadmap.md` in the TBD-subtitles style; program records follow the Reforger restructure style.

**Consequences:** The standards adapt those documents and drop everything particular to them. The
conventions survey in `documentation/program/research/` maps what was adopted.

**Supersedes:** none.

### 2026-10-02 — Local git, Conventional Commits straight to main (D11)

**Context:** One owner, one machine, agents doing the slices.

**Decision:** Local git, Conventional Commits straight to `main`, explicit paths, a Co-Authored-By
trailer, no remote yet.

**Consequences:** No branches. During a program only the orchestrator commits; agents report their files.
The rules are in the [commit conventions](/documentation/standards/commit_conventions.md).

**Supersedes:** none.

### 2026-10-02 — Panics abort and no panic path exists (D12)

**Context:** A plugin runs inside TeamSpeak and an extension inside Arma; an unwinding panic across their
C ABI is undefined behaviour, and any panic takes the host down.

**Decision:** `panic = "abort"` in dev and release. The panic-prone lints are denied (`unwrap_used`,
`expect_used`, `panic`, `indexing_slicing`, `todo`, `unimplemented`, `dbg_macro`), so no panic path exists
in production code.

**Consequences:** Errors are values to the host boundary; slices are read with `get`, tokens with length
checks, sample arithmetic saturates. Tests keep unwinding, because cargo ignores `panic` for the test
harness, and clippy allows unwrap, expect, indexing and panic in tests. Held by
`cargo clippy --workspace --all-targets -- -D warnings`.

**Supersedes:** none.

### 2026-10-02 — dist builds into target/dist and nothing installs (D13)

**Context:** The owner's TeamSpeak plugins folder and mod folders hold the live installation; a script
writing there could break it or leave stale DLLs.

**Decision:** `cargo xtask dist` builds into target/dist/tfar_1_0/ and target/dist/tfar_0_9/. No script
copies anything into a mod folder or into TeamSpeak; the owner copies by hand.

**Consequences:** dist renames the artifacts to their final names (`libtfar_plugin_<v>_linux_amd64.so`,
`task_force_radio_pipe_x64.dll`) and copies `radio-sounds/` from `reference/` without modifying it. The
manual install runbook holds the copy steps.

**Supersedes:** none.

### 2026-10-02 — Opus sub-agents run the slices, the orchestrator gates and commits (D14)

**Context:** Each milestone is too big for one session and splits into independent slices.

**Decision:** Opus sub-agents work one slice each from briefs and prompts under `documentation/program/`;
the orchestrator reviews and gates every slice, and `progress.md` holds the handoff.

**Consequences:** Every brief follows the [agent brief template](/documentation/standards/templates/agent_brief.md)
and the shared brief; the fast stage gate runs per slice and the full `cargo xtask ci ci-local` at each
milestone close.

**Supersedes:** none.

### 2026-10-02 — The 1.0 tree is an upstream clone, guarded by a manifest (D15)

**Context:** The 1.0 tree is a git clone of upstream with its own history (411 MB); the 0.9.12 tree is a
release with no version control (22 MB).

**Decision:** `reference/tfar_1_0` is a clean upstream clone (`origin/master` at `fce977fa`) and keeps
its own `.git`. The outer repository gitignores it, and only `MANIFEST.sha256` guards it, excluding
`.git/`. `reference/tfar_0_9_12` has no other version control, so it is committed as plain files.

**Consequences:** A fresh clone of the outer repository restores the 1.0 tree by cloning upstream and
checking out `fce977fa` ([reference README](/reference/README.md)). `cargo xtask verify
reference-untouched` detects drift in either tree.

**Supersedes:** none.

### 2026-10-02 — The licence is decided later (D16)

**Context:** Upstream is APL-SA; the port is new code but follows upstream behaviour closely.

**Decision:** M0 sets `license = "UNLICENSED"`, and cargo-deny ignores private crates. The IRCAM HRIR data
is credited in assets/hrir/CREDITS.md.

**Consequences:** The licence is an open question in the roadmap, settled before any release.

**Supersedes:** none.

### 2026-10-02 — The toolchain is pinned to Rust 1.99.0 (D17)

**Context:** Every gate must run on the same compiler, on the host and for the Windows target.

**Decision:** Pin 1.99.0, the newest stable on 2026-09-28, in `rust-toolchain.toml`, with `rustfmt`,
`clippy` and the targets `x86_64-unknown-linux-gnu` and `x86_64-pc-windows-gnu`.

**Consequences:** `rust-version = "1.99"` in the workspace package and `msrv = "1.99"` in clippy's
configuration. A toolchain bump is a new decision entry.

**Supersedes:** none.

### 2026-10-02 — Use libraries, don't reinvent the wheel (D18)

**Context:** Most of the port's needs (TeamSpeak bindings, filters, HRTF, SQLite, HTTP, framing,
concurrency) have maintained crates; writing them again costs time and adds bugs.

**Decision:** Product code is pure Rust and uses libraries wherever one fits; C and C++ appear only as a
test oracle (D9). The library map, checked on crates.io on 2026-10-02:

| Need | Crate (version) | Used in | We still write |
|---|---|---|---|
| TS3 SDK API 26 bindings | `ts3plugin-sys` 0.5.0 | `tfar_ts3_host` | the export layer, C-string ownership, thread separation |
| Arma extension | none fits | `tfar_arma_extension_abi` | `RVExtension`, `RVExtensionVersion`, truncating writes |
| Win32 calls | `windows-sys` 0.61.2 | `tfar_arma_extension_abi` | — |
| Vector maths | `glam` 0.33.12 | `tfar_geometry` and above | domain helpers (underwater distance, Arma angles) |
| Butterworth design | `sci-rs` 0.4.1 | `tfar_dsp_filters` | per-setup wrappers, multi-channel state |
| RBJ coefficients, DF2T | `biquad` 0.6.0 | `tfar_dsp_filters` | — |
| HRTF | `hrtf` 0.9.0 + IRCAM sphere | `tfar_hrtf` | the block adapter, per-source state |
| FFT, if needed | `rustfft` 6.4.1, `realfft` 3.5.0 | `tfar_hrtf` | — |
| Compressor | none usable | `tfar_dynamics` | a feed-forward compressor |
| Distance curve, cone | none | `tfar_spatial_audio` | the piecewise curve and the cone gain |
| WAV | `hound` 3.5.1 | `tfar_playback_mixer` | 48 kHz, 16-bit, stereo validation and cache |
| SQLite | `rusqlite` 0.40.2 (`bundled`) | `tfar_ts3_settings_db` | — |
| HTTP | `ureq` 3.4.2 (rustls) | `tfar_web_requests` | the update check and Piwik calls |
| Version compare | `version-compare` | `tfar_web_requests`, the 1.0 settings | — |
| Link framing | `postcard` 1.1.3 + `serde` 1.0.229 | `tfar_game_link_protocol` | the u32 length prefix |
| Concurrency | `parking_lot` 0.12.5, `crossbeam-channel` 0.5.17, `arc-swap` 1.9.2, `rtrb` 0.4.0 | plugins, link, audio | — |
| Logging | `tracing` 0.1.44 + `tracing-subscriber` 0.3.23 | everywhere | a layer forwarding to TeamSpeak's `logMessage` |
| Profiling | `profiling` 1.0.18 + `tracy-client` 0.19.0 | plugins | — |
| Debug window | `eframe`/`egui` 0.36.2 | `tfar_debug_window` | the panels |
| Errors | `thiserror` 2.0.21; `anyhow` 1.0.104 in tools only | all | — |
| Mock TeamSpeak host | `libloading` 0.9.0 | `tfar_ts3_mock` | the fake function table |
| xtask | `clap`, `xshell`, `cargo_metadata`, `toml_edit`, `sha2`, `walkdir`, `ignore` | tools/xtask | the gates |

Rejected: `bincode` (3.0.0 is a dead placeholder); the high-level `ts3plugin` wrapper (a global mutex with
`unwrap`, missing callbacks); `arma-rs` (no raw-string handler); `fundsp` and `idsp` (no Butterworth
band-pass); `audionimbus` (C++ Steam Audio).

**Consequences:** Every external crate is pinned once in `[workspace.dependencies]` and stays in the
crates the map names (`cargo xtask verify dependency-firewalls`). A new need gets a row here, in a new
entry, before its crate is added.

**Supersedes:** none.

### 2026-10-02 — The library map gains libc for TeamSpeak memory and libloading for the probe (D18)

**Context:** The plugin hands TeamSpeak its `infoData` text, and TeamSpeak hands it back to
`ts3plugin_freeMemory` as a bare pointer, so `tfar_ts3_host` allocates it with the C runtime's
`malloc` and releases it with `free`, which needs no size. The mock TeamSpeak host plays the other
side: it hands the plugin `malloc`ed strings and lists that the plugin returns through `freeMemory`.
The library map of the entry "Use libraries, don't reinvent the wheel (D18)" names no crate for that.
The extension probe loads the extension DLL under wine and calls `RVExtension` through its export
address, the same need the mock TeamSpeak host has for plugins.

**Decision:** Two rows join the library map:

| Need | Crate (version) | Used in | We still write |
|---|---|---|---|
| C allocator for memory that crosses the TeamSpeak boundary | `libc` 0.2.190 (no default features) | `tfar_ts3_host`, `tfar_ts3_mock` | the copy into the `malloc` block |
| Loading the extension DLL | `libloading` 0.9.0 | `tfar_extension_probe` | the `RVExtension` call and its checks |

**Consequences:** `libc` is firewalled to `tfar_ts3_host` and `tfar_ts3_mock`, and `libloading` to
`tfar_ts3_mock` and `tfar_extension_probe` (`cargo xtask verify dependency-firewalls`);
`tfar_extension_probe` joins the unsafe allow-list for its raw-pointer calls
(`cargo xtask verify unsafe-allowlist`).

**Supersedes:** none.

### 2026-10-02 — dist names the plugins task_force_radio_linux_amd64.so and TFAR_linux_amd64.so

**Context:** TeamSpeak derives a plugin's name from its file name, and the original receivers accept
plugin commands only from senders whose name starts with `task_force_radio` (0.9.12,
`reference/tfar_0_9_12/ts/src/plugin.cpp:3116`) or `TFAR` (1.0). The names in the entry "dist builds
into target/dist and nothing installs (D13)", `libtfar_plugin_<v>_linux_amd64.so`, match neither, so
Windows players running the originals would ignore our plugin commands and the reverse.

**Decision:** `cargo xtask dist` names the 0.9 plugin `task_force_radio_linux_amd64.so` and the 1.0
plugin `TFAR_linux_amd64.so`. The extension keeps `task_force_radio_pipe_x64.dll`, and everything else
in D13 holds.

**Consequences:** The two Rust plugins have distinct file names, so both can sit in one TeamSpeak
plugins folder. `tools/xtask/src/commands/dist.rs` holds the names and the install runbook names the
files.

**Supersedes:** the artifact names in 2026-10-02 — dist builds into target/dist and nothing installs (D13).

### 2026-10-03 — The library map gains url, winit and fastrand (D18)

**Context:** Three external crates joined the workspace with their slices and have no row in the
library map of the entry "Use libraries, don't reinvent the wheel (D18)": `tfar_web_requests` builds
the Piwik tracking URLs with `url`; `tfar_debug_window` needs winit's X11 event-loop option
`with_any_thread`, which eframe does not re-export, so its loop runs off TeamSpeak's main thread; the
radio effects draw the underwater sample dropouts from a seeded generator, one per effect. `libc`
already has its row in the entry "The library map gains libc for TeamSpeak memory and libloading for
the probe (D18)".

**Decision:** Three rows join the library map:

| Need | Crate (version) | Used in | We still write |
|---|---|---|---|
| Tracking URLs and their query strings | `url` 2.5.8 (`std`) | `tfar_web_requests` | the Piwik parameters |
| The X11 event loop off the main thread | `winit` 0.30.13 (`x11`) | `tfar_debug_window` | — |
| Seeded per-effect random numbers | `fastrand` 2.5.0 (no default features) | `tfar_radio_effects_0_9`, `tfar_radio_effects_1_0` | the dropout rule of each version |

**Consequences:** `url` is firewalled to `tfar_web_requests`, `winit` to `tfar_debug_window` and
`fastrand` to the two radio effect crates (`cargo xtask verify dependency-firewalls`); `libc` stays
firewalled to `tfar_ts3_host` and `tfar_ts3_mock`.

**Supersedes:** none.

### 2026-10-03 — dist follows CARGO_TARGET_DIR and ships the HRIR sphere in tfar_hrir/

**Context:** Agents build into their own `target-<id>/` folders, so a `dist` that always passes
`target/` as `--target-dir` rebuilds everything into the folder the host hook guards. Both plugins
render binaural audio with the IRCAM sphere in `assets/hrir/`, which no TeamSpeak install holds.

**Decision:** `cargo xtask dist` builds into the effective target folder, `CARGO_TARGET_DIR` when set
and not empty, else `target/`, and writes `<target>/dist/tfar_0_9/` and `<target>/dist/tfar_1_0/`. Each
version folder also gets `tfar_hrir/IRC_1002_C.bin` and `tfar_hrir/CREDITS.md`, so the folder sits
beside `radio-sounds/` in the TeamSpeak plugins folder and both plugins load the sphere from
`<plugins folder>/tfar_hrir/IRC_1002_C.bin`.

**Consequences:** The install runbook copies `tfar_hrir/` with `radio-sounds/`; everything else in
D13 holds, and nothing is copied into TeamSpeak or a mod folder.

**Supersedes:** the fixed target/dist path of 2026-10-02 — dist builds into target/dist and nothing installs (D13).

### 2026-10-10 — The 0.9.12 tree is local only, like the 1.0 tree (D15)

**Context:** The repository goes to GitHub as `darkforce09/Taskforceradio`. The 0.9.12 tree's nested
`.gitattributes` files mark its images and models as Git LFS, and its committed pointer stubs reference
objects that exist nowhere, so GitHub refuses the push (GH008).

**Decision:** `.gitignore` lists `/reference/tfar_0_9_12/`, and the history is rewritten so no commit
holds the tree; `reference/README.md` and `reference/MANIFEST.sha256` stay tracked. The commit ids cited
in the documents are mapped to the rewritten ones.

**Consequences:** A fresh clone restores both reference trees by hand
([reference README](/reference/README.md#restore-tfar_0_9_12)); `cargo xtask verify
reference-untouched` still guards both trees from the manifest.

**Supersedes:** the 2026-10-02 rule that commits `reference/tfar_0_9_12` as plain files (D15).

### 2026-10-10 — Deliverables live in their version category; no apps folder (D7)

**Context:** A top-level `apps/` folder beside `crates/` splits each mod version in two: its plugin
and extension in one place, the libraries they link in another. Every crate is a crate, so the owner
wants all of them under `crates/`, grouped by version.

**Decision:** The four deliverables move into their version category:
`crates/version_0_9/tfar_plugin_0_9`, `crates/version_0_9/tfar_extension_0_9`,
`crates/version_1_0/tfar_plugin_1_0` and `crates/version_1_0/tfar_extension_1_0`, each declaring that
category. There is no `apps/` folder and no `apps` category. Package names, crate types, tiers and the
edge laws stay as they were: the gates know the four by name (`DELIVERABLES` in
`tools/xtask/src/layout.rs`), so the plugin and extension rows of the edge matrix, the ban on depending
on a deliverable and the `app` README kind are keyed by crate, not by folder.

**Consequences:** The member globs are `crates/*/*` and `tools/*`; `cargo xtask verify crate-tiers`,
`readme-sections` and `new-crate` read the deliverables list. Paths in documents name the new folders,
and tests that climb to the repository root climb three folders.

**Supersedes:** the 2026-10-02 rule that puts the deliverables in apps/ (D7).
