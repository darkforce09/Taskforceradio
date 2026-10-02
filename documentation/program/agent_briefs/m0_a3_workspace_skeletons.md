**Status:** live

# M0 · A3 · Root configuration and all crate skeletons

You are agent A3, the workspace builder for the TFAR Rust port. Read `/documentation/program/agent_briefs/shared_brief.md` first. Then read `/documentation/standards/` and its `templates/`, written by A1 just before you.

## Goal
A compiling workspace of all 38 crates, with exactly the crate list, edges, tiers and categories in the
shared brief. Each crate is a correct skeleton: manifest, README, `src/README.md`, crate header, `prelude.rs`,
`error.rs`, and one placeholder module with one sibling test. The skeletons hold no product logic, but each
crate root states its future public surface in its README "Public surface" section.

## Steps
1. **Root `Cargo.toml`.**
   - A `[workspace]` block with `resolver = "3"` and members `"apps/*"`, `"crates/*/*"`, `"tools/*"`.
   - `[workspace.package]`: `version = "0.1.0"`, `edition = "2024"`, `rust-version = "1.99"`, `license = "UNLICENSED"`, `publish = false`.
   - `[workspace.dependencies]`:
     - every internal crate as a path dependency, grouped under `# apps`, `# crates/<category>:` and `# tools` banners;
     - every external crate from plan §4a pinned to the listed version, with `default-features = false` where sensible and a why comment;
     - `version-compare`: look up its current version with `cargo search version-compare --limit 1`.
   - `[workspace.lints.rust]`: `unsafe_code = "deny"`, `missing_docs = "warn"`, `unreachable_pub = "warn"`, `rust_2018_idioms = { level = "warn", priority = -1 }`, `unsafe_op_in_unsafe_fn = "deny"`.
   - `[workspace.lints.clippy]`:
     - `pedantic = { level = "warn", priority = -1 }`;
     - deny `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo`, `dbg_macro`, `unimplemented`, `undocumented_unsafe_blocks`;
     - allow `module_name_repetitions` and `must_use_candidate`, each with a reason comment.
   - Profiles:
     - `[profile.dev]`: `panic = "abort"`, `opt-level = 1`.
     - `[profile.dev.package."*"]`: `opt-level = 2`.
     - `[profile.release]`: `panic = "abort"`, `lto = "thin"`, `codegen-units = 1`, `strip = true`.
     - Add a comment that tests still unwind, because cargo ignores `panic` for the test harness.
2. **Root config files.**
   - `clippy.toml`: `allow-unwrap-in-tests = true`, `allow-expect-in-tests = true`, `allow-indexing-slicing-in-tests = true`, `allow-panic-in-tests = true`, `msrv = "1.99"`.
   - `rustfmt.toml`: `edition = "2024"`, `style_edition = "2024"`.
   - `typos.toml`:
     - exclude `reference/`, `target/`, `Cargo.lock`, `assets/`;
     - allow TFAR words such as "Arma", "tangent", "speakers", "Multiplifier", "Leve", "RBJ", "HRIR", "Nkey", "Dedmen" and the original C++ identifiers quoted in the specs.
   - `deny.toml`:
     - graph targets `x86_64-unknown-linux-gnu` and `x86_64-pc-windows-gnu`;
     - `[licenses]`: allow-list MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-3.0, CC0-1.0, MPL-2.0, BSL-1.0, with `confidence-threshold = 0.93`;
     - `[licenses.private] ignore = true`;
     - `[bans]`: `multiple-versions = "warn"` (it becomes deny once the real dependencies land);
     - `[sources]`: `unknown-registry = "deny"`, `unknown-git = "deny"`;
     - `[advisories]`: `unused-ignored-advisory = "deny"`.
   - `.editorconfig`:
     - `[*]`: utf-8, lf, final newline, trim trailing whitespace;
     - `*.rs`: 4 spaces;
     - `*.{toml,json,yml,yaml}`: 2 spaces;
     - `*.md`: no trimming;
     - `reference/**`: unset (all checks off).
3. **Every crate** in the shared brief's table, at its exact path.
   - **`Cargo.toml`:**
     - a header comment (1–3 lines: what it is and who calls it);
     - `[package]` with `name`, `description` and `*.workspace = true`;
     - `[package.metadata.layout]` with `category`, `tier` and `targets`;
     - `[dependencies]` listing **only the workspace edges** from the table, each with a why comment. External crates are added in their own milestone; don't add them now;
     - `[lints] workspace = true`.
   - **Library crates:**
     - `src/lib.rs` (≤ 80 lines): the four-point `//!` header (Role / Position / Signals and state / Invariants), `mod` lines and `pub use`;
     - `src/prelude.rs`, `src/error.rs` (a `thiserror` `Error` enum with one variant and `pub type Result<T>`);
     - add `thiserror` as a workspace external dependency: it is the one external allowed in M0;
     - one placeholder module (e.g. `src/placeholder.rs` exporting a documented `const CRATE_ROLE: &str`), with `#[cfg(test)] #[path = "tests/placeholder.rs"] mod tests;` and a sentence-named test;
     - `README.md` (crate root kind) and `src/README.md`.
   - **Apps:**
     - `crate-type = ["cdylib", "rlib"]` and `[lib] name` = the crate name;
     - `#![allow(unsafe_code)]` with a reason line at the crate root (export glue arrives later), with no `unsafe` code yet;
     - extension apps compile on Linux too, so their logic can be tested on the host.
   - **Tools:**
     - `tools/tfar_ts3_mock` is a lib + bin.
     - `tools/tfar_extension_probe` is a bin whose `main` compiles on both targets.
     - `tools/tfar_golden_vectors` is a lib with no build script yet.
     - **Do not create `tools/xtask`**: A4 owns it. Add it to the members only through the `tools/*` glob, which picks it up once it exists.
4. **Area READMEs:** `apps/README.md`, `crates/README.md` (the category table, the edge matrix, and the "where does a new crate go" rule), `crates/<category>/README.md` (each one), and `tools/README.md` (listing xtask as A4's).
5. **Assets:** `assets/README.md` and `assets/hrir/README.md`. These state that the IRCAM HRIR sphere arrives in M4 with `CREDITS.md`. Don't download anything.
6. **Builds:**
   - `cargo build --workspace`
   - `cargo build -p tfar_extension_0_9 -p tfar_extension_1_0 -p tfar_extension_probe --target x86_64-pc-windows-gnu`
   - Confirm the DLL and EXE files exist under `target/x86_64-pc-windows-gnu/debug/`.
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
   - `cargo deny check`
   - `typos`
   - `cargo fmt --all --check`
   - Each command runs alone and logs to `target/agent-logs/`.

## Owned files
`/Cargo.toml`, `/Cargo.lock`, `/clippy.toml`, `/rustfmt.toml`, `/typos.toml`, `/deny.toml`, `/.editorconfig`,
`/apps/**`, `/crates/**`, `/tools/README.md`, `/tools/tfar_ts3_mock/**`, `/tools/tfar_extension_probe/**`,
`/tools/tfar_golden_vectors/**`, `/assets/**`. Leave `/rust-toolchain.toml`, `/.cargo/config.toml`, `/.gitignore` and
`/.gitattributes` as they are; report needed changes as findings.

## Report additions
Tier table as built (crate → tier); any edge you had to question.

Budget: M (250k tokens). Stop there and report what is done and what is not.
