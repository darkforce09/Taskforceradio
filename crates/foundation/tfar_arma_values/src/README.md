# Arma values source

The source of `tfar_arma_values`: the crate's role constant, its error type and its prelude. The
crate's own code is not written yet.

## Contents

```text
crates/foundation/tfar_arma_values/src/
├── crate_role.rs  `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs       `Error`, the `thiserror` enum, and `Result`
├── lib.rs         the crate root: module header, `mod` lines and re-exports
├── prelude.rs     `CRATE_ROLE` for glob import
└── tests/         unit tests of `CRATE_ROLE`
```

## How it works

`lib.rs` declares the private modules `crate_role` and `error` and re-exports `CRATE_ROLE`, `Error`
and `Result` at the crate root; `prelude.rs` re-exports `CRATE_ROLE` only, so a glob import never
shadows `std`'s `Result`. `crate_role.rs` attaches its sibling test file `tests/crate_role.rs` with
`#[path]`.

## Public surface

- `CRATE_ROLE`, `Error`, `Result` and `prelude`, re-exported at the crate root.

## Boundaries

- Depends on: `thiserror`.
- Used by: `tfar_antennas_1_0`, `tfar_game_commands_0_9`, `tfar_game_commands_1_0`,
  `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_plugin_commands_0_9`, `tfar_plugin_commands_1_0`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` (`cargo xtask verify test-placement`).
