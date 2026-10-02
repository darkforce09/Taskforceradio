**Status:** live

# M0 · A4 · `tools/xtask`: gates, CI lane, dist, scaffolding, reference manifest

You are agent A4, the tooling author for the TFAR Rust port. Read `/documentation/program/agent_briefs/shared_brief.md` first. Then read `/documentation/program/laws_and_gates.md` (A2) and `/documentation/standards/` (A1). These are the specs your gates enforce. Where they disagree with this prompt, follow them and report the difference.

## Goal
`cargo xtask` is the single Rust entry point for every repository task. No shell, Python or Makefiles.

## Commands
- `cargo xtask verify <gate>` runs one gate; `cargo xtask verify all` runs every gate.
- **Exit codes:** 0 held, 1 violation, 2 could not run. A gate that could not run never reports success.
- **Output:** one line per violation, `path:line: gate: message`.

Gates:

| Gate | What it checks |
|---|---|
| `crate-tiers` | Uses `cargo_metadata`. Every manifest under apps/crates/tools is a member. Every member has `[package.metadata.layout]` with `category`, `tier` and `targets`. The path equals category + name, and the package name equals the folder name. The declared tier equals 1 + the highest workspace-dependency tier, or 0 with none. The category edge matrix holds, as do the version_0_9 ↔ version_1_0 ban, the extension-app restriction, tools never depending on apps, and dev-dependencies never pointing at apps. |
| `dependency-firewalls` | Each external crate appears only in its allowed crates (plan §4 list). |
| `crate-anatomy` | Every library has: `lib.rs` ≤ 80 lines containing only the header, attributes, `mod`, `pub use` and `pub mod`; `prelude.rs`; `error.rs` with `thiserror`; no `anyhow` in a library; `[lints] workspace = true`; edition, rust-version, license and publish from the workspace; every dependency `workspace = true`; only the allowed features (`test_fixtures` anywhere; `tracy` and `api_profiler` only in plugin apps). |
| `unsafe-allowlist` | `#![allow(unsafe_code)]` appears only in `tfar_ts3_host`, `tfar_arma_extension_abi`, the 4 apps and `tfar_ts3_mock`, always with a reason comment. Every `unsafe {` block has a `// SAFETY:` comment right above it. |
| `file-length` | Production `.rs` ≤ 500 lines. Test files (anything under a `tests/` folder) ≤ 1000. |
| `test-placement` | No inline `mod tests {` block in production files. Test modules must use `#[path = "tests/...")]`. |
| `module-headers` | Crate roots, and production files of 80+ lines, start with the four-point `//!` header. |
| `readme-coverage` | Every folder under apps, crates, tools, documentation and assets has a README.md, except `tests`, `target` and dot-folders. Its `## Contents` `text` tree lists exactly the direct children. |
| `readme-sections` | Section order per README kind, from `/documentation/standards/readme_standard.md`. |
| `status-lines` | Every Markdown file under `documentation/` starts with `**Status:** live`, `**Status:** frozen record (YYYY-MM-DD)` or `**Status:** archived`. Code READMEs have no status line. |
| `markdown-placement` | `.md` files only under `documentation/`, as `README.md` anywhere, or as the root `CLAUDE.md`/`AGENTS.md`/`README.md`. `reference/` is ignored. |
| `link-check` | Repository-root links `(/...)` and backticked paths that look like repository paths must exist. No `../` links. |
| `editorconfig` | Final newline, LF, no trailing whitespace (except in `.md`), and indent rules on tracked text files outside `reference/`. |
| `language-bans` | No `.py`, `.sh`, `.bash`, `Makefile`, `.js` or `.ts` outside `reference/`. C/C++ sources only under tools/tfar_golden_vectors/oracle/ and tools/tfar_golden_vectors/shim/. |
| `reference-untouched` | Recompute sha256 for every file under `reference/` except `reference/README.md`, `reference/MANIFEST.sha256` and anything under `reference/tfar_1_0/.git/`, then compare to `reference/MANIFEST.sha256`. Report added, removed and changed files. |

Other commands:
- **`cargo xtask reference-manifest`** writes `reference/MANIFEST.sha256` (sorted, `sha256  relative/path` lines). Refuse to overwrite unless given `--force` with a reason.
- **`cargo xtask ci ci-local`** runs every step from plan §4 "Gate set" in order. Each step runs as its own child process with its own log under `target/ci-logs/`, prints `held`, `failed` or `not run`, and the run stops at the first failure.
  - The windows clippy step is `cargo clippy -p tfar_extension_0_9 -p tfar_extension_1_0 -p tfar_arma_extension_abi -p tfar_game_link_client -p tfar_game_link_protocol -p tfar_extension_probe --target x86_64-pc-windows-gnu -- -D warnings`.
- **`cargo xtask dist`** builds release:
  - the plugins for Linux;
  - the extensions for windows-gnu.

  It then copies into `target/dist/tfar_1_0/` and `target/dist/tfar_0_9/`:
  - `libtfar_plugin_<v>_linux_amd64.so`, renamed from `libtfar_plugin_<v>.so`;
  - `task_force_radio_pipe_x64.dll`, renamed from `tfar_extension_<v>.dll`;
  - `radio-sounds/`, read from `reference/tfar_<…>/ts/radio-sounds/`, which is never modified.

  It never writes outside `target/`.
- **`cargo xtask new-crate <category> <name>`** scaffolds a crate exactly like A3's skeletons (manifest, README, `src/README.md`, `lib.rs`, `prelude.rs`, `error.rs`, a placeholder module and its sibling test), adds the workspace dependency entry, and computes the tier.

## Implementation rules
- Use `clap`, `cargo_metadata`, `toml_edit`, `sha2`, `walkdir`, `ignore` and `anyhow`, plus `xshell` only where a child command is needed.
- Command groups live in subfolders, for example:
  - `src/commands/verify/<gate>.rs`
  - `src/commands/ci/`
  - `src/commands/dist.rs`
  - `src/commands/new_crate/`
  - `src/commands/reference_manifest.rs`
- Shared layout constants (categories, the edge matrix, firewalls, the unsafe allow-list) go in one module, `src/layout.rs`, that mirrors `laws_and_gates.md`.
- **Every gate has a sibling test file** with fixture trees built in a temp dir. Each test shows the gate holding on a clean fixture and failing on a perturbed one. Required perturbations:
  - an inline `mod tests {}`;
  - a `version_0_9` → `version_1_0` dependency;
  - a wrong tier;
  - a firewall break;
  - a missing README;
  - a changed byte under a fixture `reference/`;
  - a `.py` file;
  - a missing status line.
- The xtask crate follows the workspace conventions (manifest metadata `category = "tools"`, tier 0, README, `src/README.md`, the four-point headers, sibling tests). As a binary it may use `anyhow`.

## Steps at the end
1. `cargo xtask reference-manifest`: create the manifest. Before that, confirm `reference/` is still byte-identical, using the scratchpad lists `/tmp/claude-1000/-run-media-system-Disk-2-Projects-Taskforceradio/19487e80-757a-431f-981a-ae1156f78b05/scratchpad/pre_1_0.sha` and `pre_0_9.sha`; their format is `sha256sum` output relative to each tree root.
2. `cargo xtask verify all`. Fix violations in your own files. Report violations in others' files as findings and do not edit them.
3. Three live perturbation proofs on the real repo, restored right after:
   - plant an inline test in a placeholder;
   - add a `tfar_game_commands_1_0` dependency to `tfar_game_commands_0_9`;
   - append a byte to a copy-then-restore of one reference file. Back up the file first, then restore it and confirm with `cmp`.
4. `cargo xtask ci ci-local`.
5. `cargo xtask dist`: confirm the six artifacts and the two `radio-sounds/` trees exist with their final names.

## Owned files
`/tools/xtask/**` and `/reference/MANIFEST.sha256` (created by the command). You may add `tools/xtask`-only
entries to `[workspace.dependencies]` in `/Cargo.toml`: external crates under a `# tools/xtask` banner, and nothing else.

## Report additions
A gate table: | Gate | Held on the repo | Perturbation proven |, plus the full `ci-local` step table.

Budget: L (350k tokens). Stop there and report what is done and what is not.
