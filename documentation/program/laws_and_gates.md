**Status:** live

# Laws and gates

The structural laws of the workspace and the gate set that enforces them: the category edge matrix,
the external-crate firewalls, the unsafe allow-list, the feature rules, every gate with its exit
codes, the difference between the stage gate and the milestone gate, and the perturbation proofs
each gate must carry. The xtask tool mirrors this file in one layout module; when the two disagree,
this file is the spec and the code is fixed. Gates are run as `cargo xtask verify <gate>`; the whole
set runs as `cargo xtask ci ci-local`.

## Tiers

Every member declares `[package.metadata.layout]` with three keys:

```toml
[package.metadata.layout]
category = "crates/audio"   # the parent path: crates/<category>, apps or tools
tier = 1                    # 0 with no workspace dependency, else 1 + the highest dependency tier
targets = "any"             # "windows" only for tfar_extension_probe
```

The path of a crate is its category plus its name, and the package name equals the folder name.
Normal and build dependency edges point strictly down the tiers. The authoritative crate list with
every edge and tier is in the [shared brief](/documentation/program/agent_briefs/shared_brief.md);
the [crate catalogue](/documentation/program/crate_catalogue.md) repeats the tiers.

## Category edge matrix

A crate may depend on a workspace crate only if the target's category is allowed for its own.

| From | May depend on | Never on |
|---|---|---|
| crates/foundation | crates/foundation | every other category |
| crates/audio | crates/foundation, crates/audio | everything else |
| crates/teamspeak | crates/foundation, crates/teamspeak | everything else |
| crates/game_link | crates/foundation, crates/game_link | everything else |
| crates/services | crates/foundation | everything else |
| crates/interface | crates/foundation | everything else |
| crates/version_0_9 | crates/foundation, crates/audio, crates/teamspeak, `tfar_game_link_protocol` (the only game_link crate), crates/services, crates/interface, crates/version_0_9 | **crates/version_1_0**, apps, tools |
| crates/version_1_0 | crates/foundation, crates/audio, crates/teamspeak, `tfar_game_link_protocol` (the only game_link crate), crates/services, crates/interface, crates/version_1_0 | **crates/version_0_9**, apps, tools |
| apps: `tfar_plugin_0_9` | any crates/* category except crates/version_1_0 | crates/version_1_0, apps, tools |
| apps: `tfar_plugin_1_0` | any crates/* category except crates/version_0_9 | crates/version_0_9, apps, tools |
| apps: `tfar_extension_0_9`, `tfar_extension_1_0` | crates/foundation, crates/game_link | everything else |
| tools | any crates/* category | apps |

Rules beside the matrix:
- Foundation crates depend only on other foundation crates (`tfar_arma_values` → `tfar_geometry`).
- **version_0_9 and version_1_0 never depend on each other**, in any kind of dependency.
- Dev-dependencies are exempt from the tier order but never point at apps.
- An app never depends on another app.

## External-crate firewalls

Each external crate may appear as a direct dependency only in the crates listed. Crates not listed
here (`thiserror`, `tracing`, `parking_lot`, `crossbeam-channel`, `arc-swap`, `rtrb`, `glam`,
`serde`, `profiling`, `version-compare`) are free for libraries; `anyhow` is for tools only.

| External crate | Allowed only in |
|---|---|
| `windows-sys` | `tfar_arma_extension_abi` |
| `rusqlite` | `tfar_ts3_settings_db` |
| `ureq` | `tfar_web_requests` |
| `eframe`, `egui` | `tfar_debug_window` |
| `cc` | `tfar_golden_vectors` |
| `sci-rs`, `biquad` | `tfar_dsp_filters` |
| `hrtf`, `rustfft`, `realfft` | `tfar_hrtf` |
| `hound` | `tfar_playback_mixer` |
| `ts3plugin-sys` | `tfar_ts3_host`, `tfar_ts3_mock` |
| `postcard` | the game_link crates |
| `libloading` | `tfar_ts3_mock`, `tfar_extension_probe` |
| `tracy-client` | the two plugin apps, behind the `tracy` feature |
| `anyhow` | tools only; never a library or an app |
| `clap`, `xshell`, `cargo_metadata`, `toml_edit`, `sha2`, `walkdir`, `ignore` | `xtask` |

## Unsafe allow-list

- `unsafe_code = "deny"` is a workspace lint. Only these crates may lift it, each with
  `#![allow(unsafe_code)]` at the crate root and a reason comment on the line above:
  - `tfar_ts3_host` (TeamSpeak C ABI);
  - `tfar_arma_extension_abi` (Arma C ABI and Win32 calls);
  - the 4 apps (export glue only);
  - `tfar_ts3_mock` (loads plugins and fakes the function table).
- Every `unsafe` block carries a `// SAFETY:` comment directly above it
  (`clippy::undocumented_unsafe_blocks` is denied).
- `unsafe_op_in_unsafe_fn` is denied, so an `unsafe fn` body still needs explicit blocks.
- Panics never cross the FFI boundary: the profiles set `panic = "abort"`, and export functions
  catch nothing because nothing in production code may panic.

## Feature rules

- `test_fixtures` is the only feature a library may declare, and it is enabled only from
  `[dev-dependencies]`.
- `tracy` and `api_profiler` exist only in the two plugin apps; they forward to the `profiling`
  crate's backends.
- No other feature names exist in workspace crates. External crates are declared with
  `default-features = false` in `[workspace.dependencies]` wherever their defaults are not needed.

## Gates

Exit codes are the same for every gate: **0** the gate held, **1** a violation was found, **2** the
gate could not run (a missing input, an unreadable file, a tool not installed). A gate that could
not run never reports success. Violations print one per line as `path:line: gate: message`.

### Toolchain steps (in `ci-local` order)

| Step | Command | What it checks |
|---|---|---|
| fmt | `cargo fmt --all --check` | Formatting per the root rustfmt config |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | Workspace lints and the clippy deny list on the host target |
| clippy-windows | `cargo clippy -p tfar_extension_0_9 -p tfar_extension_1_0 -p tfar_arma_extension_abi -p tfar_game_link_client -p tfar_game_link_protocol -p tfar_extension_probe --target x86_64-pc-windows-gnu -- -D warnings` | The extension side compiles and lints clean for Windows |
| test | `cargo test --workspace` | Every unit and integration test |
| deny | `cargo deny check` | Licence allow-list, bans, sources, advisories |
| typos | `typos` | Spelling outside the reference trees |

A toolchain step exits with the tool's own status; `ci-local` maps a non-zero status to "failed"
and a missing tool to "not run" (exit 2).

### Repository gates (`cargo xtask verify <gate>`)

| Gate | What it checks | Could not run (2) when |
|---|---|---|
| `crate-tiers` | Every manifest under apps, crates and tools is a member; layout metadata present; path = category + name; package name = folder name; declared tier = computed tier; the category edge matrix; the version ban; extension apps limited to foundation and game_link; tools never on apps; dev-dependencies never on apps | `cargo metadata` fails |
| `dependency-firewalls` | Each external crate appears only in its allowed crates (table above) | `cargo metadata` fails |
| `crate-anatomy` | `lib.rs` ≤ 80 lines holding only the header, attributes, `mod`, `pub mod` and `pub use`; `prelude.rs`; `error.rs` with a `thiserror` error; no `anyhow` in a library; `[lints] workspace = true`; edition, rust-version, license and publish from the workspace; every dependency `workspace = true`; only allowed features | A manifest does not parse |
| `unsafe-allowlist` | `#![allow(unsafe_code)]` only in the allow-listed crates, each with a reason; every `unsafe {` with a `// SAFETY:` comment above | A source file is unreadable |
| `file-length` | Production `.rs` ≤ 500 lines; files under a tests folder ≤ 1000 | A source root is missing |
| `test-placement` | No inline `mod tests {` in production files; test modules attach with `#[path = "tests/<file>.rs"]` | A source root is missing |
| `module-headers` | Crate roots and production files of 80+ lines start with the four-point `//!` header (Role / Position / Signals and state / Invariants) | A source root is missing |
| `readme-coverage` | Every folder under apps, crates, tools, documentation and assets has a README.md (except `tests`, `target` and dot-folders), and its `## Contents` `text` tree lists exactly the direct children | The tree walk fails |
| `readme-sections` | Section order per README kind, from the README standard (planned: documentation/standards/readme_standard.md) | The standard's kind table is missing |
| `status-lines` | Every Markdown file under documentation starts with `**Status:** live`, `**Status:** frozen record (YYYY-MM-DD)` or `**Status:** archived`; code READMEs have none | The tree walk fails |
| `markdown-placement` | `.md` files only under documentation, as a README.md anywhere, or as the root CLAUDE.md, AGENTS.md and README.md; `reference/` ignored | The tree walk fails |
| `link-check` | Repository-root links (targets starting with a slash) and backticked repository paths exist; no parent-relative links | The tree walk fails |
| `editorconfig` | Final newline, LF, no trailing whitespace outside `.md`, indent rules, on tracked text files outside `reference/` | `git ls-files` fails |
| `language-bans` | No `.py`, `.sh`, `.bash`, Makefile, `.js` or `.ts` outside `reference/`; C/C++ sources only in the oracle and shim folders of `tfar_golden_vectors` | The tree walk fails |
| `reference-untouched` | sha256 of every file under `reference/` (except its README, the manifest itself and the 1.0 tree's `.git` folder) matches the reference manifest; reports added, removed and changed files | The manifest is missing or unreadable |

`cargo xtask verify all` runs every repository gate and exits with the worst status.

## Stage gate and milestone gate

| | Stage gate (fast, per slice) | Milestone gate (full, at close) |
|---|---|---|
| Who runs it | The agent before its report, and the orchestrator before each slice commit | The orchestrator at the milestone close |
| What runs | `cargo fmt --all --check`; `cargo clippy -p <touched crates> --all-targets -- -D warnings`; `cargo test -p <touched crates>`; the documentation gates (`readme-coverage`, `readme-sections`, `status-lines`, `markdown-placement`, `link-check`); the structural gate for the slice's subject (`crate-tiers` when manifests change) | `cargo xtask ci ci-local`: every toolchain step and every repository gate, in order, stopping at the first failure |
| Extra | The Windows clippy step when an extension-side crate changes | The milestone's extra gates and owner checkpoints in the [program plan](/documentation/program/program_plan.md#5-milestones-and-agents) |

`ci-local` runs each step as its own child process with its own log in the ci-logs folder under
target, and prints `held`, `failed` or `not run` per step.

## Perturbation proofs

Every gate has a sibling test that builds a fixture tree in a temporary folder, shows the gate
holding on the clean fixture, and failing on a perturbed copy. These perturbations are required:

| Perturbation | Gate that must fail |
|---|---|
| An inline `mod tests {}` in a production file | `test-placement` |
| A `version_0_9` → `version_1_0` dependency | `crate-tiers` |
| A wrong declared tier | `crate-tiers` |
| A firewalled crate in a crate not allowed to use it | `dependency-firewalls` |
| A folder without a README.md | `readme-coverage` |
| A changed byte under a fixture `reference/` | `reference-untouched` |
| A `.py` file | `language-bans` |
| A documentation file without a status line | `status-lines` |

At the M0 close three proofs also run live on the real repository, each restored right after:
1. a planted inline test in a placeholder module fails `test-placement`;
2. a `tfar_game_commands_1_0` dependency added to `tfar_game_commands_0_9` fails `crate-tiers`;
3. one byte appended to a reference file (backed up first, restored and confirmed with `cmp`)
   fails `reference-untouched`.

Every check added in a later milestone gets its own perturbation proof in the slice that adds it.
