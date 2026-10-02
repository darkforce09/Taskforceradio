# Xtask source

The source of the `xtask` binary: the command line, the command groups, the laws as data, and the
readers the commands share.

## Contents

```text
tools/xtask/src/
├── cli.rs      the `clap` command line: every subcommand and its arguments
├── commands/   one module or folder per command group: verify, ci, dist, new-crate, reference-manifest
├── layout.rs   the laws as data: categories, edge matrix, firewalls, unsafe allow-list, limits
├── main.rs     the crate root: parses the command line, runs one command, forms the exit code
├── support/    the shared readers: repository listing, workspace members, Rust, Markdown, READMEs
└── tests/      the fixture trees every gate test builds, and the layout table tests
```

## How it works

`main.rs` parses `cli.rs` and hands the [`Repository`](#public-surface) of this workspace to one
command under `commands/`. The gates under `commands/verify/` never read the laws from anywhere but
`layout.rs`, and never touch the file system but through `support/`, so a test can point a gate at
a fixture tree in a temporary folder. `tests/fixture_tree.rs` builds that tree: a small workspace
of three crates, written with the `new-crate` templates, with its READMEs, a README standard and a
reference tree with its manifest, on which every gate holds.

## Public surface

- The binary only; every module is `pub(crate)`. The commands are listed in the crate README.

## Boundaries

- Depends on: `anyhow`, `clap`, `cargo_metadata`, `toml_edit`, `sha2`, `walkdir`, `xshell`.
- Used by: nothing in the workspace links it; people and agents run it through `cargo xtask`.
- Rules: production files stay within 500 lines and carry the four-point header
  (`cargo xtask verify file-length`, `cargo xtask verify module-headers`); tests live in sibling
  `tests/` folders (`cargo xtask verify test-placement`).
