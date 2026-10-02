# Mock host

`MockHost`: a started plugin inside the fake TeamSpeak, with the drivers that play server,
client and audio events into it.

## Contents

```text
tools/tfar_ts3_mock/src/mock_host/
├── audio_drivers.rs   one voice block through each voice callback, and the custom rolloff
├── event_drivers.rs   connect, disconnect, switch tabs, join, move, leave, talk, metadata, commands
├── host_lifecycle.rs  `MockHost`: start in TeamSpeak's order, info panel, commands, shutdown
├── mod.rs             the module lines and the public re-exports
└── tests/             an in-process stand-in plugin and the tests of start, drivers and audio
```

## How it works

`MockHost::start` refuses a plugin not built for API 26, begins the session, then calls
`setFunctionPointers`, `registerPluginID("mock_plugin_id")` and `init`; a non-zero `init` is
`Error::InitFailed`. An event driver changes the fake state under the lock, releases it and calls
the export, so the plugin's queries see the change; a callback the plugin does not export is
skipped. An audio driver copies the caller's samples, passes the counts, one speaker bit per
channel and the flags TeamSpeak passes, and returns what the plugin left; a missing export is
`Error::MissingExport`. `shutdown` runs once, from the call or the drop, before the library
unloads.

`tests/in_process_plugin.rs` defines `extern "C"` exports inside the test binary and hands them
to the host over the test executable itself, so the tests run the real start order, drivers and
memory rules without a separately built plugin.

## Boundaries

- Depends on: `fake_state`, `fake_functions` (the table), `plugin_library`; `tfar_ts3_host`,
  `ts3plugin-sys`.
- Used by: the plugin apps' integration tests and the binary.
- Rules: start order (`start_sets_the_table_then_registers_the_id_then_runs_init`); the state is
  set before each callback (`clients_join_move_and_leave_with_the_state_set_before_each_callback`);
  an `infoData` block goes back to `ts3plugin_freeMemory`
  (`info_data_copies_the_text_and_hands_the_block_back`).
