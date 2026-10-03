# TeamSpeak function table

The safe `Ts3` handle over the `TS3Functions` table TeamSpeak hands the plugin at load: one method
per SDK function TFAR calls, grouped by what the functions do.

## Contents

```text
crates/teamspeak/tfar_ts3_host/src/ts3_functions/
├── client_actions.rs      moving, muting, plugin commands to other clients, wave playback
├── client_variables.rs    client, own-client, channel and server variables, read and written
├── connection_queries.rs  server tabs, connection status, our client ID, client and channel lists
├── handle.rs              `Ts3`: the table, set once, and the plugin ID, latest wins; error codes into `Ts3Error`
├── mod.rs                 the module lines and the `Ts3` re-export
├── owned_memory.rs        copying TeamSpeak strings and lists, freeing them with `freeMemory`
├── plugin_folder.rs       `Ts3::plugin_folder`: `getPluginPath`, else the loaded library's folder
├── sound_and_paths.rs     3D sound, the pre-processor, plugin and config paths, log and chat tab
└── tests/                 the methods against a fake table, the memory helpers, the plugin folder
```

## How it works

`handle.rs` keeps the table in a process-wide `OnceLock<Ts3>` that `ts3plugin_setFunctionPointers`
fills, and the plugin ID in an `ArcSwapOption` inside it that every `ts3plugin_registerPluginID`
replaces: TeamSpeak's "Reload" keeps the library (and the table) loaded but registers a new ID, and
`getPluginPath` and `sendPluginCommand` work only with the current one. Every
method file adds an `impl Ts3` block: it turns text into a `CString` (refusing a NUL byte with
`Ts3Error::InteriorNul`), calls the function pointer, and passes TeamSpeak's code through
`Ts3::check`, which treats `ERROR_ok` and `ERROR_ok_no_update` as success and otherwise asks
`getErrorMessage` for the text of a `Ts3Error::Call`. A string or zero-terminated list TeamSpeak
allocates goes through `owned_memory.rs`, which copies it and calls `freeMemory` exactly once.
Paths come from a zeroed 4096-byte buffer read up to its first NUL. `plugin_folder.rs` resolves
the folder of the plugin's files: `getPluginPath` first, and when that is empty or fails, the
canonical folder of the loaded plugin `.so`, which `dladdr` names for a function of this crate; it
logs which of the two answered. An optional 3D vector (the
listener's forward and up) is passed as a null pointer when absent, as the originals do. No method takes a lock, so no
TeamSpeak call ever runs under one; none may run on an audio thread.

## Boundaries

- Depends on: `ts3plugin-sys` (`Ts3Functions`, the SDK enums), `arc-swap` (the plugin ID),
  `libc` (`dladdr`), the crate's `host_ids`, `host_values` and `error`; TeamSpeak's function
  table at run time.
- Used by: `plugin_exports` (installs the table, registers the ID, reads callback strings through
  `owned_memory`), `logging` (`log_message`, `print_message_to_current_tab`), and the plugin apps
  through `Ts3`.
- Rules: every TeamSpeak allocation is freed once after it is copied
  (`tests/ts3_calls.rs`, `tests/owned_memory.rs`); every call that passes the plugin ID passes the
  one registered last (`tests/ts3_calls.rs`); the plugin folder is never empty while the library's
  folder is known (`tests/plugin_folder.rs`); every slot used sits at its SDK header offset
  (`src/tests/sdk_layout.rs`); every `unsafe` block names who owns the memory
  (`cargo xtask verify unsafe-allowlist`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  TeamSpeak functions the 1.0 plugin calls (§4).
- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  TeamSpeak functions the 0.9.12 plugin calls (§4).
