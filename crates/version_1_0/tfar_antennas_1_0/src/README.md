# Antennas for 1.0 source

The source of `tfar_antennas_1_0`: the radio tower, the tower registry and its connection search,
the net id and the error type.

## Contents

```text
crates/version_1_0/tfar_antennas_1_0/src/
├── antenna.rs             `Antenna`: one tower, its reach and the loss of a hop through it
├── antenna_connection.rs  `AntennaConnection`, `NO_CONNECTION_LOSS`, `loss_or_no_connection`
├── antenna_manager.rs     `AntennaManager`: add, remove and find the best tower
├── crate_role.rs          `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs               `Error` (a tower with an invalid range) and `Result`
├── lib.rs                 the crate root: module header, `mod` lines and re-exports
├── net_id.rs              `NetId`: the Arma `creator:object` id of a tower
├── prelude.rs             the public names for glob import
└── tests/                 unit tests of each module
```

## How it works

`AntennaManager` keeps a `Vec<Antenna>` in insertion order; `find_connection` scans it once,
keeping the tower of lowest loss below 1 among those both ends reach, and returns it with its loss
as an `AntennaConnection` (a copy of the tower, so no reference outlives the caller's lock).
`Antenna::new` refuses a negative or non-finite range, which the original stores but can never
connect through.

## Public surface

- `Antenna`, `AntennaManager`, `AntennaConnection`, `NetId`, `NO_CONNECTION_LOSS`,
  `loss_or_no_connection`, `Error`, `Result`, `CRATE_ROLE` and `prelude`, re-exported at the
  crate root.

## Boundaries

- Depends on: `tfar_geometry`, `tfar_arma_values`, `thiserror`.
- Used by: `tfar_plugin_1_0`, `tfar_radio_routing_1_0`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` (`cargo xtask verify test-placement`).
