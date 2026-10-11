# TFAR 1.0

Everything of the 1.0 mod:

- [`tfar_plugin_1_0`](/crates/version_1_0/tfar_plugin_1_0/README.md) — the TeamSpeak plugin (a
  Linux `.so`): the 0.9 set plus the async queue, antennas, `SETCFG`, the debug window and the
  focus-aware channel move.
- [`tfar_extension_1_0`](/crates/version_1_0/tfar_extension_1_0/README.md) — the Arma extension (a
  Windows DLL): `~` async calls, `DFRAME` → `NEEDCFG`/`OK`, `MISSIONEND`, `FOCUS`, on port 47820.
- [`tfar_plugin_logic_1_0`](/crates/version_1_0/tfar_plugin_logic_1_0/README.md) — the plugin's
  safe logic: settings, antennas, commands, client state, radio effects, routing, voice pipeline.

```text
Arma 3 -> tfar_extension_1_0 -- TCP 127.0.0.1:47820 --> tfar_plugin_1_0 <- TeamSpeak 3
inside the plugin: plugin_settings -> game_commands -> client_state -> radio_routing (+ antennas) -> voice_pipeline
```

```bash
cargo test -p tfar_plugin_logic_1_0 -p tfar_plugin_1_0 -p tfar_extension_1_0
cargo build -p tfar_extension_1_0 --target x86_64-pc-windows-gnu
```

A 1.0 crate never depends on a 0.9 crate; the extension uses only foundation and game link; nothing
depends on a deliverable (`cargo xtask verify crate-tiers`). `cargo xtask dist` gives the built
files the names the mod and TeamSpeak expect.
