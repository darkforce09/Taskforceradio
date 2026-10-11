# Geometry

The `tfar_geometry` crate: positions, directions and distances in Arma's frame, built on glam. Used
by `tfar_antennas_1_0`, `tfar_arma_values`, `tfar_client_state_0_9`, `tfar_client_state_1_0`,
`tfar_game_commands_0_9`, `tfar_game_commands_1_0`, `tfar_hrtf`, `tfar_plugin_0_9`,
`tfar_plugin_1_0`, `tfar_radio_routing_0_9`, `tfar_radio_routing_1_0`, `tfar_spatial_audio`.

## How it works

The vector and angle helpers every positional crate shares, over `glam`'s `DVec3`: `Position` and
`Direction` in Arma's frame (x east, y north, z up, metres), `Angle` in radians, the underwater
distance rule and the rotation about the vertical axis. Constructors reject non-finite coordinates
with `Error::NonFinite`; every direction is normalised to unit length or kept at zero.

`Position` and `Direction` wrap a `glam::DVec3` in Arma's frame (x east, y north, z up, metres).
Their constructors from `[f64; 3]` reject NaN and infinity with `Error::NonFinite`.
`Direction::from_vector` normalises with `glam`'s `normalize_or_zero`, so every direction has length
1 or is zero. `Position::direction_to` builds on it. `Direction::view_angle` (`atan2(x, y)`, the
compass heading) and `Direction::polar_angle` (`atan2(y, x)`) return an `Angle`, which
`normalized_signed` wraps into (-π, π]. `rotate_about_z` applies the 2D rotation matrix to a
position, and `distance_underwater` clips a segment at z = 0. Every module attaches its sibling test
file in `tests/` with `#[path]`.

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
- `Error` (`NonFinite`), `Result` and `prelude`.

## Boundaries

- Depends on: no workspace crate; `glam` for the vectors and `thiserror` for the error type.
- Used by: `tfar_antennas_1_0`, `tfar_arma_values`, `tfar_client_state_0_9`,
  `tfar_client_state_1_0`, `tfar_game_commands_0_9`, `tfar_game_commands_1_0`, `tfar_hrtf`,
  `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_radio_routing_0_9`, `tfar_radio_routing_1_0`,
  `tfar_spatial_audio`.
- Rules: every direction it returns has unit length or is the zero vector, never NaN. Tier 0 in
  `crates/foundation`: every workspace dependency has a lower tier and follows the category edge matrix (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
