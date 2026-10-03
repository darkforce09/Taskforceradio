# Radio routing for 1.0 source

The source of `tfar_radio_routing_1_0`: the reception rules, the mute rules, the effect error
levels, the underwater range and the serious channel lookup.

## Contents

```text
crates/version_1_0/tfar_radio_routing_1_0/src/
├── crate_role.rs       `CRATE_ROLE`: what the crate is for, in one sentence
├── effect_error.rs     the error level of a radio effect from distance and antenna loss
├── error.rs            `Error` and `Result`
├── lib.rs              the crate root: module header, `mod` lines and re-exports
├── mute/               `mute_decision`, `MuteTracker`, `killed_mute_matrix`
├── mute.rs             the module lines and re-exports of `mute/`
├── prelude.rs          the public names for glob import
├── reception/          `is_over_radio(_into)` and its intercom, carried-radio and speaker rules
├── reception.rs        the module lines and re-exports of `reception/`
├── remote_tangent.rs   `apply_remote_tangent`: the session's tangent handling with this routing
├── routing_context.rs  `RoutingContext`: the session, settings, towers and time a rule reads
├── serious_mode.rs     `serious_mode_channel` and `ChannelId`
├── tests/              unit tests of the top-level modules and the shared test world
└── underwater.rs       `diver_radio_distance` and `underwater_weighted_distance`
```

## How it works

`reception/` turns a `RoutingContext` and a sender into `HeardVia` values; `mute/` decides mutes
on top of it; `effect_error.rs` and `underwater.rs` hold the formulas the voice pipeline shares
with the rules. `remote_tangent.rs` wires `is_over_radio` into the client state's tangent
handling, which cannot depend on this crate.

## Public surface

- Every name the crate root re-exports, listed in the crate README.

## Boundaries

- Depends on: `tfar_client_state_1_0`, `tfar_antennas_1_0`, `tfar_plugin_settings_1_0`,
  `tfar_geometry`, `tfar_arma_values`, `thiserror`.
- Used by: `tfar_plugin_1_0`, `tfar_voice_pipeline_1_0`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` (`cargo xtask verify test-placement`).
