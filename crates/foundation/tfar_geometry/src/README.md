# Geometry source

The source of `tfar_geometry`: positions, unit directions, angles, the underwater distance rule and
the rotation about the vertical axis, each in its own module.

## Contents

```text
crates/foundation/tfar_geometry/src/
├── angle.rs                `Angle`: radians, degree conversion and the (-π, π] wrap
├── crate_role.rs           `CRATE_ROLE`: what the crate is for, in one sentence
├── direction.rs            `Direction`: a unit vector or zero, with its view and polar angles
├── error.rs                `Error` (`NonFinite`), the `thiserror` enum, and `Result`
├── lib.rs                  the crate root: module header, `mod` lines and re-exports
├── position.rs             `Position`: a point in metres, with distance and direction to another
├── prelude.rs              the public types and functions for glob import
├── rotation.rs             `rotate_about_z`: a position turned about the z axis
├── tests/                  unit tests, one file per module
└── underwater_distance.rs  `distance_underwater`: the submerged length of a straight segment
```

## How it works

`Position` and `Direction` wrap a `glam::DVec3` in Arma's frame (x east, y north, z up, metres).
Their constructors from `[f64; 3]` reject NaN and infinity with `Error::NonFinite`.
`Direction::from_vector` normalises with `glam`'s `normalize_or_zero`, so every direction has length
1 or is zero. `Position::direction_to` builds on it. `Direction::view_angle` (`atan2(x, y)`, the
compass heading) and `Direction::polar_angle` (`atan2(y, x)`) return an `Angle`, which
`normalized_signed` wraps into (-π, π]. `rotate_about_z` applies the 2D rotation matrix to a
position, and `distance_underwater` clips a segment at z = 0. Every module attaches its sibling test
file in `tests/` with `#[path]`.

## Public surface

- `Position`: `ORIGIN`, `from_array`, `from_dvec3`, `as_dvec3`, `to_array`, `x`, `y`, `z`,
  `distance_to`, `direction_to`.
- `Direction`: `ZERO`, `from_vector`, `from_array`, `as_dvec3`, `to_array`, `x`, `y`, `z`,
  `is_zero`, `view_angle`, `polar_angle`.
- `Angle`: `ZERO`, `from_radians`, `from_degrees`, `radians`, `to_degrees`, `cos`, `sin`,
  `normalized_signed`, and `+`, `-`, unary `-`.
- `distance_underwater(from, to)` and `rotate_about_z(position, angle)`.
- `Error`, `Result`, `CRATE_ROLE` and `prelude`, re-exported at the crate root.

## Boundaries

- Depends on: `glam` (`DVec3` and its normalisation); `thiserror`.
- Used by: `tfar_antennas_1_0`, `tfar_arma_values`, `tfar_client_state_0_9`,
  `tfar_client_state_1_0`, `tfar_game_commands_0_9`, `tfar_game_commands_1_0`, `tfar_hrtf`,
  `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_radio_routing_0_9`, `tfar_radio_routing_1_0`,
  `tfar_spatial_audio`.
- Rules: a direction has unit length or is zero, never NaN (`tests/direction.rs`);
  `normalized_signed` never leaves (-π, π] (`tests/angle.rs`); production files hold no `unwrap`,
  `expect`, `panic` or indexing (clippy deny list); tests live in `tests/`
  (`cargo xtask verify test-placement`).
