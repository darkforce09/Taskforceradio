# TFAR 1.0 TeamSpeak plugin

The `tfar_plugin_1_0` crate: the TeamSpeak 3 plugin for the 1.0 mod, built as a native Linux shared
library. TeamSpeak 3 loads it from its plugins folder.

## Contents

```text
apps/tfar_plugin_1_0/
├── Cargo.toml  the `tfar_plugin_1_0` package (cdylib + rlib): tier 6, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The TeamSpeak plugin for the 1.0 mod: the `ts3plugin_*` exports, the game link listener with its
async queue, the TeamSpeak event handlers, antennas, `SETCFG`, the focus-aware channel move and the
debug window, wired to the 1.0 crates. The code is not written yet: `src/lib.rs` carries the module
header and re-exports, `src/crate_role.rs` states the crate's role in one constant, and
`src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_plugin_1_0   # the host unit tests
cargo build -p tfar_plugin_1_0  # libtfar_plugin_1_0.so under target/debug/
```

## Configuration

It reads the `-tfdebug` launch switch (debug port 47821 instead of 47820), the settings the game
sends with `SETCFG`, and TeamSpeak's config path for its own files. The features `tracy` and
`api_profiler`, both off by default, switch on the Tracy and API profilers.

## Exports

- Symbols: none yet, because the code is not written. The crate is for the `ts3plugin_*` exports
  TeamSpeak resolves at load time (name, version, API version, init and shutdown, and the event
  callbacks).
- Artifact: libtfar_plugin_1_0_linux_amd64.so, which `cargo xtask dist` renames from
  libtfar_plugin_1_0.so into target/dist/tfar_1_0/ beside radio-sounds/.
- Installed by: the owner, by hand, into the Flatpak TeamSpeak plugins folder
  (~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/); TeamSpeak then lists the plugin in its
  plugin settings.

## Boundaries

- Depends on: `tfar_ts3_host` (the safe TeamSpeak export layer and function table);
  `tfar_ts3_settings_db` (TeamSpeak settings lookups); `tfar_game_link_server` (the plugin-side link
  listener); `tfar_web_requests` (the update check and Piwik calls); `tfar_voice_pipeline_1_0` (the
  1.0 voice frame processing); `tfar_radio_routing_1_0` (the 1.0 reception decisions);
  `tfar_client_state_1_0` (the 1.0 per-client state); `tfar_game_commands_1_0` (the typed 1.0 game
  commands); `tfar_plugin_commands_1_0` (the 1.0 plugin command and metadata formats);
  `tfar_playback_mixer` (radio sound playback); `tfar_geometry` (positions, directions and distances);
  `tfar_arma_values` (the parsers for Arma's field, array and position strings);
  `tfar_plugin_settings_1_0` (the 1.0 settings); `tfar_antennas_1_0` (the 1.0 antenna and tower
  model); `tfar_debug_window` (the debug window); `thiserror` for the error type.
- Used by: TeamSpeak 3, which loads the shared library; `tools/tfar_ts3_mock` in the plugin test
  suites.
- Rules: a plugin whose game link never connected this session stays dormant and passes audio
  through untouched. Tier 6 in `apps`: every workspace dependency has a lower tier and follows the
  edge matrix in `crates/README.md` (`cargo xtask verify crate-tiers`). `unsafe` is allowed here for
  the export glue only (`cargo xtask verify unsafe-allowlist`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
- [Windows dependencies](/documentation/program/behaviour_specs/windows_dependencies.md) — the Win32
  objects the originals use and what replaces them.
