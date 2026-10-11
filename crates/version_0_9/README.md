# TFAR 0.9.12

Everything of the 0.9.12 mod:

- [`tfar_plugin_0_9`](/crates/version_0_9/tfar_plugin_0_9/README.md) — the TeamSpeak plugin (a
  Linux `.so`): exports, threads, the game link server, push-to-talk, metadata.
- [`tfar_extension_0_9`](/crates/version_0_9/tfar_extension_0_9/README.md) — the Arma extension (a
  Windows DLL): every call is a synchronous round trip to the plugin on port 47810.
- [`tfar_plugin_logic_0_9`](/crates/version_0_9/tfar_plugin_logic_0_9/README.md) — the plugin's
  safe logic: commands, client state, radio effects, routing, voice pipeline.

```text
Arma 3 -> tfar_extension_0_9 -- TCP 127.0.0.1:47810 --> tfar_plugin_0_9 <- TeamSpeak 3
inside the plugin: game_commands -> client_state -> radio_routing -> voice_pipeline (+ radio_effects, tfar_audio)
```

```bash
cargo test -p tfar_plugin_logic_0_9 -p tfar_plugin_0_9 -p tfar_extension_0_9
cargo build -p tfar_extension_0_9 --target x86_64-pc-windows-gnu
```

A 0.9 crate never depends on a 1.0 crate; the extension uses only foundation and game link; nothing
depends on a deliverable (`cargo xtask verify crate-tiers`). `cargo xtask dist` gives the built
files the names the mod and TeamSpeak expect.
