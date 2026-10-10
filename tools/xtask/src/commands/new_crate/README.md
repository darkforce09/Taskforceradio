# New crate

`cargo xtask new-crate`: scaffolds a crate in the standard anatomy, adds its
`[workspace.dependencies]` entry and lists it in its category's README, so the new crate passes
every gate before its first line of code.

## Contents

```text
tools/xtask/src/commands/new_crate/
├── crate_templates.rs      every file of the skeleton: manifest, READMEs, lib.rs, prelude, error, role
├── mod.rs                  the module list
├── parent_readme_entry.rs  the new crate's line in its category README's Contents block
├── scaffold.rs             the command: checks, the computed tier, the writes
├── tests/                  tests that a scaffolded crate and app pass every gate, and of the refusals
└── workspace_entry.rs      the path entry in the root `[workspace.dependencies]`, in its category group
```

## How it works

`scaffold.rs` normalises the category (`audio` becomes `crates/audio`), checks the name
(lowercase with underscores, the `tfar_` prefix, the version suffix in a version category), reads
the members through `cargo metadata`, checks every requested dependency edge against the edge
matrix and computes the tier: 0 with no dependency, else 1 plus the highest. Only then does it
render the files with `crate_templates.rs` and write them, with the root manifest edited by
`workspace_entry.rs` (placed after the last entry of the same category, keeping every comment) and
the category README by `parent_readme_entry.rs` (entries redrawn in name order). An app (one of
the four deliverables, known by name) is a `cdylib` + `rlib` with `#![allow(unsafe_code)]` under a
reason comment and an Exports section; a library or tool is a plain library. The test fixtures build
their crates with the same templates.

## Commands

Run as `cargo xtask new-crate` from the repository root.

### new-crate

- Synopsis: `cargo xtask new-crate <CATEGORY> <NAME> [--summary <SUMMARY>] [--depends-on <CRATES>]`
- Does: writes `<category>/<name>/` with `Cargo.toml`, `README.md`, `src/README.md`, `src/lib.rs`,
  `src/prelude.rs`, `src/error.rs`, `src/crate_role.rs` and `src/tests/crate_role.rs`; adds the
  path entry to the root manifest and the folder line to the category README. `--depends-on` takes
  a comma-separated list of workspace crates.
- Exit codes: 0 written; 1 refused (a bad name or category, an existing crate, an illegal edge)
  with nothing written.
- Example: `cargo xtask new-crate audio tfar_noise_gate --summary "A noise gate for the voice path." --depends-on tfar_sample_buffer`

## Boundaries

- Depends on: `support/cargo_workspace.rs` for the members and tiers, `support/readme_span.rs` for
  the Contents block, `layout.rs` for the edge matrix, `toml_edit`.
- Used by: `main.rs`; the gate fixtures in `tools/xtask/src/tests/fixture_tree.rs` use
  `crate_templates.rs`.
- Rules:
  - a scaffolded crate and a scaffolded app pass every gate (`tests/scaffold.rs`);
  - an illegal edge is refused before anything is written
    (`an_illegal_edge_is_refused_before_anything_is_written`).

## Related documentation

- [Coding standards](/documentation/standards/coding_standards.md#crate-anatomy) — the anatomy
  the templates write.
- [Laws and gates](/documentation/program/laws_and_gates.md#category-edge-matrix) — the edges the
  command checks.
