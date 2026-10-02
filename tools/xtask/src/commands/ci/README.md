# CI lane

The `cargo xtask ci` lanes. `ci-local` is the milestone gate: every toolchain step and every
repository gate, in order, each as its own child process with its own log.

## Contents

```text
tools/xtask/src/commands/ci/
├── ci_local.rs  the runner: one child per step, its log, held, failed or not run, the stop
├── ci_steps.rs  the step table: fmt, clippy, clippy-windows, test, deny, typos, then every gate
├── mod.rs       the module list
└── tests/       tests of the step order and of the runner's outcomes and logs
```

## How it works

`ci_steps.rs` lists the steps; `ci_local.rs` runs them from the repository root, one at a time,
with stdout and stderr in `target/ci-logs/<nn>-<step>.log` behind a `$ <command>` header line, and
prints one line per step. The gates run as this same binary (`verify <gate>`), so a gate's
"could not run" exit 2 is told apart from a violation. The first step that does not hold stops the
lane; the steps after it print `not run` and get no log.

```text
fmt ─► clippy ─► clippy-windows ─► test ─► deny ─► typos ─► crate-tiers ─► … ─► reference-untouched
 │
 └── each: target/ci-logs/<nn>-<step>.log, held | failed | not run, stop at the first non-held
```

## Commands

Run as `cargo xtask ci <lane>` from the repository root.

### ci-local

- Synopsis: `cargo xtask ci ci-local`
- Does: runs `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  the windows clippy step (`cargo clippy -p tfar_extension_0_9 -p tfar_extension_1_0
  -p tfar_arma_extension_abi -p tfar_game_link_client -p tfar_game_link_protocol
  -p tfar_extension_probe --target x86_64-pc-windows-gnu -- -D warnings`), `cargo test --workspace`,
  `cargo deny check`, `typos`, then `cargo xtask verify <gate>` for every gate in the verify table.
- Exit codes: 0 every step held; 1 a step failed; 2 a step could not run (a missing tool, or a
  gate exiting 2).
- Example: `cargo xtask ci ci-local`

## Boundaries

- Depends on: the verify gate table for the gate names; at run time cargo, `cargo-deny` and
  `typos` on the `PATH`.
- Used by: the orchestrator at every milestone close; anyone before a large change.
- Rules:
  - the step order matches the toolchain steps of the laws document, then every gate
    (`tests/ci_steps.rs`);
  - a missing tool is `not run`, never `held`, and the lane stops at the first failure
    (`tests/ci_local.rs`).

## Related documentation

- [Laws and gates](/documentation/program/laws_and_gates.md) — the toolchain steps and the stage and
  milestone gates.
