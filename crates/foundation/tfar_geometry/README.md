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

The vector and angle helpers every positional crate shares, over `glam`'s `DVec3`: `Position` and
`Direction` in Arma's frame (x east, y north, z up, metres), `Angle` in radians, the underwater
distance rule and the rotation about the vertical axis. Constructors reject non-finite coordinates
with `Error::NonFinite`; every direction is normalised to unit length or kept at zero. The modules
are described in `src/README.md`.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_geometry  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

- `Position`: a point in metres, built from `[f64; 3]`, with `distance_to` and `direction_to`.
- `Direction`: a unit vector or zero, from `from_vector` or `from_array`, with `view_angle`
  (`atan2(x, y)`) and `polar_angle` (`atan2(y, x)`).
- `Angle`: radians, with `from_degrees`, `to_degrees` and the (-π, π] wrap `normalized_signed`.
- `distance_underwater(from, to)`: the length of a segment below z = 0.
- `rotate_about_z(position, angle)`: a position turned about the z axis.
- `Error` (`NonFinite`), `Result`, `CRATE_ROLE` and `prelude`.

## Boundaries

- Depends on: no workspace crate; `glam` for the vectors and `thiserror` for the error type.
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
