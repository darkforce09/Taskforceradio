# TFAR 0.9.12 crates

The behaviour of the 0.9.12 mod: its game commands, plugin commands, radio effects, client state,
reception routing and voice pipeline.

## Contents

```text
crates/version_0_9/
├── tfar_client_state_0_9/     tier 3: the 0.9 per-client state
├── tfar_game_commands_0_9/    tier 2: the typed 0.9 game commands
├── tfar_plugin_commands_0_9/  tier 2: the 0.9 plugin command and metadata formats
├── tfar_radio_effects_0_9/    tier 2: the 0.9 radio effect chains
├── tfar_radio_routing_0_9/    tier 4: the 0.9 reception decisions
└── tfar_voice_pipeline_0_9/   tier 5: the 0.9 voice frame processing
```

## How it works

A game message is parsed by `tfar_game_commands_0_9` and updates `tfar_client_state_0_9`. On each
voice frame, `tfar_voice_pipeline_0_9` asks `tfar_radio_routing_0_9` how the listener hears the
speaker, then applies `tfar_radio_effects_0_9` and the shared audio crates.
`tfar_plugin_commands_0_9` encodes what clients tell each other through TeamSpeak.

```text
game_commands -> client_state -> radio_routing -> voice_pipeline
radio_effects ---^                                ^-- audio crates
```

## Getting started

Run from the repository root:

```bash
# the unit tests of every crate in this category
cargo test \
  -p tfar_client_state_0_9 \
  -p tfar_game_commands_0_9 \
  -p tfar_plugin_commands_0_9 \
  -p tfar_radio_effects_0_9 \
  -p tfar_radio_routing_0_9 \
  -p tfar_voice_pipeline_0_9
```

## Boundaries

- Depends on: foundation, audio, teamspeak, `tfar_game_link_protocol`, services and interface
  crates, and the version_0_9 category itself.
- Used by: `tfar_plugin_0_9`.
- Rules: a version_0_9 crate never depends on a version_1_0 crate, and the reverse
  (`cargo xtask verify crate-tiers` holds the edges, `cargo xtask verify dependency-firewalls` the
  external crates); each crate declares `category = "crates/version_0_9"` and its tier: 0 without a
  workspace dependency, else one above the highest.

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the original plugin this track ports.
