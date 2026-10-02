# Antennas for 1.0

The `tfar_antennas_1_0` crate: the 1.0 antenna and radio tower model used for terrain-aware
reception. Used by `tfar_plugin_1_0`, `tfar_radio_routing_1_0`.

## Contents

```text
crates/version_1_0/tfar_antennas_1_0/
├── Cargo.toml  the `tfar_antennas_1_0` package (lib): tier 2, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The 1.0 antenna model: radio towers and their positions and ranges, and the antenna connection that
lets a signal hop past terrain. The code is not written yet: `src/lib.rs` carries the module header
and re-exports, `src/crate_role.rs` states the crate's role in one constant, and `src/error.rs`
holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_antennas_1_0  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `Antenna` and `AntennaRegistry`.
- `find_antenna_connection`: the best path between two radios through the antennas.

## Boundaries

- Depends on: `tfar_geometry` (positions, directions and distances); `tfar_arma_values` (the parsers
  for Arma's field, array and position strings); `thiserror` for the error type.
- Used by: `tfar_plugin_1_0`, `tfar_radio_routing_1_0`.
- Rules: a connection is only found through antennas within their range. Tier 2 in
  `crates/version_1_0`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
