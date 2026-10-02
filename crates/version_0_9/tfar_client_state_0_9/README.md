# Client state for 0.9

The `tfar_client_state_0_9` crate: the per-client state of the 0.9.12 plugin: positions, radios,
tangents and effect state. Used by `tfar_plugin_0_9`, `tfar_radio_routing_0_9`,
`tfar_voice_pipeline_0_9`.

## Contents

```text
crates/version_0_9/tfar_client_state_0_9/
├── Cargo.toml  the `tfar_client_state_0_9` package (lib): tier 3, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The state the 0.9.12 plugin keeps per TeamSpeak client, keyed by real name: position, radios,
tangent state, voice volume and the effect and HRTF state of each sender. The code is not written
yet: `src/lib.rs` carries the module header and re-exports, `src/crate_role.rs` states the crate's
role in one constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_client_state_0_9  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `ClientState`: one client's position, radios, tangents and effect state.
- `ClientRegistry`: all clients keyed by real nickname, with expiry.

## Boundaries

- Depends on: `tfar_geometry` (positions, directions and distances); `tfar_game_commands_0_9` (the
  typed 0.9 game commands); `tfar_radio_effects_0_9` (the 0.9 radio effect chains); `tfar_dsp_filters`
  (the Butterworth and RBJ filters); `tfar_dynamics` (the compressor); `tfar_hrtf` (binaural direction
  rendering); `thiserror` for the error type.
- Used by: `tfar_plugin_0_9`, `tfar_radio_routing_0_9`, `tfar_voice_pipeline_0_9`.
- Rules: effect state is only reached while its client's lock is held. Tier 3 in
  `crates/version_0_9`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
