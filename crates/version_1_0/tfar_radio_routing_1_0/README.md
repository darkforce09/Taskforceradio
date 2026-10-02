# Radio routing for 1.0

The `tfar_radio_routing_1_0` crate: whether and how a 1.0 listener hears a speaker: direct, over
radio, intercom or speaker radio. Used by `tfar_plugin_1_0`, `tfar_voice_pipeline_1_0`.

## Contents

```text
crates/version_1_0/tfar_radio_routing_1_0/
├── Cargo.toml  the `tfar_radio_routing_1_0` package (lib): tier 4, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The 1.0 reception rules: `isOverRadio` and its neighbours decide whether a listener hears a speaker
directly, over radio (with antennas and terrain interception), over intercom or through a speaker
radio, and with which effect. The code is not written yet: `src/lib.rs` carries the module header
and re-exports, `src/crate_role.rs` states the crate's role in one constant, and `src/error.rs`
holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_radio_routing_1_0  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `Reception`: how a listener hears a speaker, if at all.
- `route_speaker`: the reception for one speaker and listener pair.

## Boundaries

- Depends on: `tfar_client_state_1_0` (the 1.0 per-client state); `tfar_antennas_1_0` (the 1.0
  antenna and tower model); `tfar_plugin_settings_1_0` (the 1.0 settings); `tfar_geometry` (positions,
  directions and distances); `thiserror` for the error type.
- Used by: `tfar_plugin_1_0`, `tfar_voice_pipeline_1_0`.
- Rules: routing reads client state and never changes it. Tier 4 in `crates/version_1_0`: every
  workspace dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
