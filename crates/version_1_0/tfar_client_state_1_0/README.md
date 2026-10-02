# Client state for 1.0

The `tfar_client_state_1_0` crate: the per-client state of the 1.0 plugin: positions, radios,
tangents, vehicles and effect state. Used by `tfar_plugin_1_0`, `tfar_radio_routing_1_0`,
`tfar_voice_pipeline_1_0`.

## Contents

```text
crates/version_1_0/tfar_client_state_1_0/
├── Cargo.toml  the `tfar_client_state_1_0` package (lib): tier 3, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The state the 1.0 plugin keeps per TeamSpeak client, keyed by real name: position, radios, tangent
state, vehicle and intercom, and the effect and HRTF state of each sender. The code is not written
yet: `src/lib.rs` carries the module header and re-exports, `src/crate_role.rs` states the crate's
role in one constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_client_state_1_0  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `ClientState`: one client's position, radios, tangents, vehicle and effect state.
- `ClientRegistry`: all clients keyed by real nickname, with expiry.

## Boundaries

- Depends on: `tfar_geometry` (positions, directions and distances); `tfar_game_commands_1_0` (the
  typed 1.0 game commands); `tfar_radio_effects_1_0` (the 1.0 radio effect chains);
  `tfar_plugin_settings_1_0` (the 1.0 settings); `tfar_dsp_filters` (the Butterworth and RBJ filters);
  `tfar_dynamics` (the compressor); `tfar_hrtf` (binaural direction rendering); `thiserror` for the
  error type.
- Used by: `tfar_plugin_1_0`, `tfar_radio_routing_1_0`, `tfar_voice_pipeline_1_0`.
- Rules: effect state is only reached while its client's lock is held. Tier 3 in
  `crates/version_1_0`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
