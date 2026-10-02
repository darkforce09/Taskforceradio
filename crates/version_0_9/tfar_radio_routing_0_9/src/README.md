# Radio routing for 0.9 source

The source of `tfar_radio_routing_0_9`: the distances, the reception rules, the mute decisions, the
serious-mode checks and the planning of a sender's sound state.

## Contents

```text
crates/version_0_9/tfar_radio_routing_0_9/src/
├── crate_role.rs    `CRATE_ROLE`: what the crate is for, in one sentence
├── distances.rs     raw and effective distance, the diver radio's reach, the effect error
├── dsp_planning.rs  `dsp_requirements`: the sound objects a sender's voice needs
├── error.rs         `Error`: why a sender's voice needs no sound state
├── heard_via.rs     `HeardVia`, `HeardOn`, `IgnoreTangents` and `LOCAL_RADIO_ID`
├── lib.rs           the crate root: module header, `mod` lines and re-exports
├── mute_decisions/  the serious-mode mutes and the `MuteTracker` of what the plugin muted
├── prelude.rs       the routing types and functions a caller imports
├── reception/       `is_over_local_radio`, `is_over_radio` and `receptions`
├── serious_mode.rs  `is_serious_mode` and the other radio plugins' channels
└── tests/           unit tests per module
```

## How it works

```text
ServerRadioData ──reception──▶ Vec<HeardVia> ──▶ voice pipeline (per frame)
       │                             ├──▶ mute_decisions ──MuteTracker──▶ MuteDecision ──▶ plugin
       │                             └──▶ dsp_planning ──▶ DspRequirements ──▶ ClientData::ensure_dsp
       └── distances (effective distance, diver reach, effect error)
```

`reception/` decides every way a sender reaches the listener, on `distances.rs`; `heard_via.rs`
holds the result type and the ignore flags. `mute_decisions/` and `dsp_planning.rs` both build on
the receptions. `serious_mode.rs` holds the channel-name checks the plugin applies to TeamSpeak's
channel names before it asks for a mute.

## Public surface

- The reception functions and types, the distances, the mute decisions with `MuteTracker`, the
  serious-mode checks, `dsp_requirements`, `Error`, `Result`, `CRATE_ROLE`, `prelude`.

## Boundaries

- Depends on: `tfar_client_state_0_9`, `tfar_geometry`, `thiserror`.
- Used by: `tfar_plugin_0_9`, `tfar_voice_pipeline_0_9`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); every
  function but the `set_*` ones takes the state by shared reference; tests live in `tests/` folders
  (`cargo xtask verify test-placement`).
