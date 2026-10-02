# Geometry

The `tfar_geometry` crate: positions, directions and distances in Arma's frame, built on glam. Used
by `tfar_antennas_1_0`, `tfar_arma_values`, `tfar_client_state_0_9`, `tfar_client_state_1_0`,
`tfar_game_commands_0_9`, `tfar_game_commands_1_0`, `tfar_hrtf`, `tfar_plugin_0_9`,
`tfar_plugin_1_0`, `tfar_radio_routing_0_9`, `tfar_radio_routing_1_0`, `tfar_spatial_audio`.

## Contents

```text
crates/foundation/tfar_geometry/
├── Cargo.toml  the `tfar_geometry` package (lib): tier 0, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The vector and angle helpers every positional crate shares: Arma world positions, view directions,
distances (including the underwater rule) and correct normalisation. The code is not written yet:
`src/lib.rs` carries the module header and re-exports, `src/crate_role.rs` states the crate's role
in one constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_geometry  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `Position` and `Direction`: Arma world coordinates and unit view vectors over `glam`.
- distance and angle helpers: listener-to-speaker distance, the underwater distance rule, Arma
  heading angles.
- a normalisation that never divides by zero.

## Boundaries

- Depends on: no workspace crate; `thiserror` for the error type. Planned externals: `glam`.
- Used by: `tfar_antennas_1_0`, `tfar_arma_values`, `tfar_client_state_0_9`,
  `tfar_client_state_1_0`, `tfar_game_commands_0_9`, `tfar_game_commands_1_0`, `tfar_hrtf`,
  `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_radio_routing_0_9`, `tfar_radio_routing_1_0`,
  `tfar_spatial_audio`.
- Rules: every direction it returns has unit length or is the zero vector, never NaN. Tier 0 in
  `crates/foundation`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
- [Library survey](/documentation/program/research/library_survey.md) — the external crates chosen
  for each need and why.
