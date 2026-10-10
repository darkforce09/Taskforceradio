# TFAR 0.9.12 crates

Everything of the 0.9.12 mod: its two deliverables, the TeamSpeak plugin and the Arma extension,
and the libraries behind them: game commands, plugin commands, radio effects, client state,
reception routing and voice pipeline.

## Contents

```text
crates/version_0_9/
├── tfar_client_state_0_9/     tier 3: the 0.9 per-client state
├── tfar_extension_0_9/        tier 2: the 0.9.12 `task_force_radio_pipe` extension DLL (deliverable)
├── tfar_game_commands_0_9/    tier 2: the typed 0.9 game commands
├── tfar_plugin_0_9/           tier 6: the 0.9.12 TeamSpeak plugin `.so` (deliverable)
├── tfar_plugin_commands_0_9/  tier 2: the 0.9 plugin command and metadata formats
├── tfar_radio_effects_0_9/    tier 2: the 0.9 radio effect chains
├── tfar_radio_routing_0_9/    tier 4: the 0.9 reception decisions
└── tfar_voice_pipeline_0_9/   tier 5: the 0.9 voice frame processing
```

## How it works

Arma calls `tfar_extension_0_9` through `callExtension`; the extension forwards each call over
localhost TCP (port 47810) to `tfar_plugin_0_9`, which TeamSpeak has loaded. In the plugin, a game
message is parsed by `tfar_game_commands_0_9` and updates `tfar_client_state_0_9`. On each voice
frame, `tfar_voice_pipeline_0_9` asks `tfar_radio_routing_0_9` how the listener hears the speaker,
then applies `tfar_radio_effects_0_9` and the shared audio crates. `tfar_plugin_commands_0_9`
encodes what clients tell each other through TeamSpeak.

```text
Arma 3 -> tfar_extension_0_9 -- TCP 127.0.0.1:47810 --> tfar_plugin_0_9 <- TeamSpeak 3

inside the plugin:
game_commands -> client_state -> radio_routing -> voice_pipeline
radio_effects ---^                                ^-- audio crates
```

The two deliverables are a cdylib plus an rlib each; `cargo xtask dist` gives the built files the
names the mod and TeamSpeak expect.

## Getting started

Run from the repository root:

```bash
# the unit and host tests of every crate in this category
cargo test \
  -p tfar_client_state_0_9 \
  -p tfar_extension_0_9 \
  -p tfar_game_commands_0_9 \
  -p tfar_plugin_0_9 \
  -p tfar_plugin_commands_0_9 \
  -p tfar_radio_effects_0_9 \
  -p tfar_radio_routing_0_9 \
  -p tfar_voice_pipeline_0_9
# the extension DLL
cargo build -p tfar_extension_0_9 --target x86_64-pc-windows-gnu
```

## Boundaries

- Depends on: the libraries on foundation, audio, teamspeak, `tfar_game_link_protocol`, services
  and interface crates and on this category; `tfar_plugin_0_9` on any crate outside version_1_0;
  `tfar_extension_0_9` on foundation and game_link crates only.
- Used by: TeamSpeak 3 and Arma 3, which load the two deliverables; the libraries by
  `tfar_plugin_0_9`.
- Rules: a version_0_9 crate never depends on a version_1_0 crate, and the reverse; no crate
  depends on a deliverable (`cargo xtask verify crate-tiers` holds the edges by crate name,
  `cargo xtask verify dependency-firewalls` the external crates); each crate declares
  `category = "crates/version_0_9"` and its tier: 0 without a workspace dependency, else one above
  the highest.

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the original plugin this track ports.
- [Arma extensions](/documentation/program/behaviour_specs/extensions.md) — the original
  extensions and the Arma ABI facts.
