# Arma values

The `tfar_arma_values` crate: parsing of the strings Arma sends: tab-split fields, Arma arrays,
frequency lists, positions. Used by `tfar_antennas_1_0`, `tfar_game_commands_0_9`,
`tfar_game_commands_1_0`, `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_plugin_commands_0_9`,
`tfar_plugin_commands_1_0`.

## Contents

```text
crates/foundation/tfar_arma_values/
├── Cargo.toml  the `tfar_arma_values` package (lib): tier 1, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The parsers for every value shape Arma's `callExtension` strings carry: tab-split fields split
exactly like the original getline loop, Arma array literals, frequency lists and positions. The code
is not written yet: `src/lib.rs` carries the module header and re-exports, `src/crate_role.rs`
states the crate's role in one constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_arma_values  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `split_fields`: tab-split fields exactly like the original getline loop, empty fields kept.
- `ArmaArray`: the parsed form of an Arma array literal.
- `FrequencyList` and position parsing, returning `Result` instead of indexing past the end.

## Boundaries

- Depends on: `tfar_geometry` (positions, directions and distances); `thiserror` for the error type.
- Used by: `tfar_antennas_1_0`, `tfar_game_commands_0_9`, `tfar_game_commands_1_0`,
  `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_plugin_commands_0_9`, `tfar_plugin_commands_1_0`.
- Rules: a parser never indexes past the tokens it was given; a short message is an `Error`, never a
  panic. Tier 1 in `crates/foundation`: every workspace dependency has a lower tier and follows the
  edge matrix in `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
