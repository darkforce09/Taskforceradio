# Radio routing for 0.9

The `tfar_radio_routing_0_9` crate: whether and how a 0.9.12 listener hears a speaker: direct, over
radio or over a speaker radio. Used by `tfar_plugin_0_9`, `tfar_voice_pipeline_0_9`.

## Contents

```text
crates/version_0_9/tfar_radio_routing_0_9/
├── Cargo.toml  the `tfar_radio_routing_0_9` package (lib): tier 4, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The 0.9.12 reception rules: `isOverLocalRadio` and `isOverRadio` decide whether a listener hears a
speaker directly, over SW, LR or DD radio or through a speaker radio, and with which effect. The
code is not written yet: `src/lib.rs` carries the module header and re-exports, `src/crate_role.rs`
states the crate's role in one constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_radio_routing_0_9  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `Reception`: how a listener hears a speaker, if at all.
- `route_speaker`: the reception for one speaker and listener pair.

## Boundaries

- Depends on: `tfar_client_state_0_9` (the 0.9 per-client state); `tfar_geometry` (positions,
  directions and distances); `thiserror` for the error type.
- Used by: `tfar_plugin_0_9`, `tfar_voice_pipeline_0_9`.
- Rules: routing reads client state and never changes it. Tier 4 in `crates/version_0_9`: every
  workspace dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
