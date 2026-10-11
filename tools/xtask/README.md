# Xtask

The `xtask` crate: the repository's single entry point for every repository task, run as
`cargo xtask`. It holds the verify gates, the local CI lane, the release copy into the dist folder,
crate scaffolding and the reference manifest, so no task needs a shell script.

## Contents

```text
tools/xtask/
├── Cargo.toml  the `xtask` package (bin): tier 0 in tools, workspace lints, the xtask-only externals
└── src/        the command line, the commands, the layout laws and the shared readers
```

## How it works

The `cargo xtask` alias in `.cargo/config.toml` runs this binary from the repository root. `main`
parses the command line (`src/cli.rs`) and runs one command from `src/commands/`. Every gate reads
the laws from one module, `src/layout.rs`, which mirrors
[laws and gates](/documentation/program/laws_and_gates.md), and reads the repository through
`src/support/`: the file list is what `git ls-files --cached --others --exclude-standard` prints,
so untracked new files are checked before their first commit and ignored build output never is.

```text
cargo xtask <command>
      │
      ▼
  main.rs ── cli.rs (clap)
      │
      ├── verify <gate>|all ──► commands/verify/  ──► layout.rs + support/
      ├── ci ci-local ────────► commands/ci/      ──► cargo, cargo-deny, typos, verify <gate>
      ├── dist ───────────────► commands/dist.rs  ──► cargo build --release, copies under target/dist/
      ├── new-crate ──────────► commands/new_crate/
      └── reference-manifest ─► commands/reference_manifest.rs
```

Every gate exits 0 when it held, 1 on a violation and 2 when it could not run, and prints one
`path:line: gate: message` line per violation.

## Getting started

Run from the repository root:

```bash
cargo xtask verify all                 # every repository gate; exit 0, 1 or 2
cargo xtask verify crate-tiers         # one gate
cargo xtask ci ci-local                # the toolchain steps, then every gate; logs in target/ci-logs/
cargo xtask dist                       # release builds copied into target/dist/tfar_1_0/ and tfar_0_9/
cargo test -p xtask                    # the gate fixtures and the command tests, a few seconds
```

## Configuration

- `CARGO`: the cargo binary the commands run (set by cargo itself; `cargo` on the `PATH`
  otherwise), read in `src/commands/ci/ci_steps.rs` and `src/commands/dist.rs`.
- `CARGO_TARGET_DIR`: honoured by every cargo child; `dist` always passes the repository's
  `target/` folder so the dist folder lands there.
- No Cargo feature and no settings file.

## Public surface

- The `cargo xtask` binary and its commands: `verify <gate>` and `verify all`, `ci ci-local`,
  `dist`, `new-crate <category> <name>`, `reference-manifest`. The gate names are cited by
  `CLAUDE.md`, the standards and the program records and stay stable.

## Boundaries

- Depends on: no workspace crate. `anyhow`, `clap`, `cargo_metadata`, `toml_edit`, `sha2`,
  `walkdir` and `xshell`; at run time git, cargo, `cargo-deny` and `typos`.
- Used by: developers, every agent's stage gate and the orchestrator's milestone gate.
- Rules:
  - every gate has a sibling fixture test that shows it holding on a clean tree and failing on a
    perturbed one (`src/commands/verify/tests/`);
  - `src/layout.rs` mirrors the laws document, and the document wins when they disagree;
  - `dist` writes only under `target/` and only reads `reference/`; `reference-manifest` writes
    only `reference/MANIFEST.sha256`.

## Related documentation

- [Laws and gates](/documentation/program/laws_and_gates.md) — every gate, its exit codes and its
  perturbation proofs.
- [README standard](/documentation/standards/code_and_documentation.md#documents) — the README rules the
  documentation gates hold.
- [Coding standards](/documentation/standards/code_and_documentation.md) — the crate anatomy and source
  rules the source gates hold.
