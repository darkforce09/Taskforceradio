# Client state for 1.0 source

The source of `tfar_client_state_1_0`: the session, the client directory and client data, the
game data, the reception vocabulary, the per-sender DSP objects and the game link state.

## Contents

```text
crates/version_1_0/tfar_client_state_1_0/src/
├── client_data.rs              `ClientData`: one client's fields, position, distance and liveness
├── client_directory.rs         `ClientDirectory`: the clients by nickname and by client id
├── crate_role.rs               `CRATE_ROLE`: what the crate is for, in one sentence
├── data_frame.rs               `DataFrame`: the game's frame counter, or invalid
├── effects/                    `ClientEffects` and its requirements, filter designs and keys
├── effects.rs                  the module lines and re-exports of `effects/`
├── error.rs                    `Error` (a missing HRIR sphere, a refused design) and `Result`
├── game_data.rs                `GameData` and `Speaker`: the local player's game state
├── game_link_state.rs          `GameLinkState`: connect, disconnect and in-game from message times
├── lib.rs                      the crate root: module header, `mod` lines and re-exports
├── plugin_enabled.rs           the 10 s plugin-enabled cache of a client
├── prelude.rs                  the public names, and the foreign types of public fields
├── radio_volume.rs             `radio_volume_gain` and `remote_radio_sound`
├── receiving_transmissions.rs  the senders the local player receives, one count each
├── reception.rs                `HeardVia` and the vocabulary of how a sender is heard
├── session/                    `SessionState` and the apply function of each message
├── session.rs                  the module lines and re-exports of `session/`
└── tests/                      unit tests of the top-level modules and their shared builders
```

## How it works

`SessionState` (in `session/`) owns a `GameData` and a `ClientDirectory`. The directory maps a
nickname to its `ClientData` in a `BTreeMap` and a client id to its nickname in a `HashMap`, and
every change goes through it so the two always agree; the local player is the entry whose id the
directory marks as own. A `ClientData` keeps its position as reported and extrapolates it by the
vehicle velocity for the time since the last `POS` (`client_position(now)`); liveness uses a
separate activity clock, which tangents also refresh. Each `ClientData` owns a `ClientEffects`
(in `effects/`). `reception.rs` holds `HeardVia`, which the routing crate produces and the tangent
handling and voice pipeline read. `GameLinkState` stands apart: one per plugin.

## Public surface

- Every name the crate root re-exports, listed in the crate README.

## Boundaries

- Depends on: `tfar_geometry`, `tfar_arma_values`, `tfar_game_commands_1_0`,
  `tfar_plugin_commands_1_0`, `tfar_radio_effects_1_0`, `tfar_plugin_settings_1_0`,
  `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`, `tfar_sample_buffer`, `thiserror`.
- Used by: `tfar_plugin_1_0`, `tfar_radio_routing_1_0`, `tfar_voice_pipeline_1_0`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` (`cargo xtask verify test-placement`); a nickname or client id changes only
  through `ClientDirectory` (`a_rename_moves_the_entry_with_its_state`).
