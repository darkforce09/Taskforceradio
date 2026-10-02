# Client state for 0.9 source

The source of `tfar_client_state_0_9`: the server and client state, the application of the game
messages and plugin commands, the data frames, the game session and each sender's sound state.

## Contents

```text
crates/version_0_9/tfar_client_state_0_9/src/
├── client_data.rs         `ClientData`: one player by nickname, the two resets, the plugin-enabled cache
├── client_id.rs           `ClientId`: a TeamSpeak client id
├── command_application/   the `apply_*` methods for every game message and plugin command
├── crate_role.rs          `CRATE_ROLE`: what the crate is for, in one sentence
├── data_frames.rs         the data frame window, lookups by client id and expiry
├── dsp_state/             the sound state a sender's voice passes through, made on request
├── error.rs               `Error`: a filter, effect or compressor that cannot be built
├── game_session.rs        `GameSession`, the game start and end
├── lib.rs                 the crate root: module header, `mod` lines and re-exports
├── nickname_list.rs       the channel client list: nickname to client id
├── prelude.rs             the state types with the command value types they carry
├── server_radio_data.rs   `ServerRadioData`: everything known on one server connection
├── speaker_radio.rs       `SpeakerRadio`: a radio on loudspeaker, and the wave line
├── tests/                 unit tests per module
└── vehicle_muffling.rs    `VehicleMuffling`: the combined isolation between two vehicles
```

## How it works

```text
GameCommand / PluginCommand ──apply_*──▶ ServerRadioData ──clients──▶ ClientData ──dsp──▶ ClientDsp
        (game link, events)                │ data_frames: has_fresh_data, remove_expired
                                           │ game_session: start_game, end_game
GameSession ◀── record_message / check_timeout (game link thread, service tick)
DspRequirements (from the routing) ──ClientData::ensure_dsp──▶ ClientDsp (made off the audio thread)
```

`server_radio_data.rs` holds the state; `command_application/` adds the methods that apply each
command and return what the plugin does next (a reply, a rename, a sound, a mute check);
`data_frames.rs` and `nickname_list.rs` add the lookups and the client list. `client_data.rs` holds
one player, whose `ClientDsp` (`dsp_state/`) owns the compressor, direct-voice and per-radio sound
objects. `game_session.rs` turns the message traffic into the in-game flag.

## Public surface

- `ServerRadioData`, `ClientData`, `ClientId`, `FrequencySettings`, `SpeakerRadio`,
  `VehicleMuffling`, `GameSession` and its transition types.
- The outcome types from `command_application/` and the sound state types from `dsp_state/`.
- `Error`, `Result`, `CRATE_ROLE`, `prelude`.

## Boundaries

- Depends on: `tfar_geometry`, `tfar_game_commands_0_9`, `tfar_radio_effects_0_9`,
  `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`, `thiserror`.
- Used by: `tfar_plugin_0_9`, `tfar_radio_routing_0_9`, `tfar_voice_pipeline_0_9`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); no
  file reads the clock; tests live in `tests/` folders (`cargo xtask verify test-placement`).
