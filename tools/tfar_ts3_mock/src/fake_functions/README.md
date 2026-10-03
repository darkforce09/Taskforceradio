# Fake TeamSpeak functions

The `extern "C"` fakes the mock puts into the `Ts3Functions` table it hands a plugin, one per
TeamSpeak function the TFAR plugins call, and the table that holds them.

## Contents

```text
tools/tfar_ts3_mock/src/fake_functions/
├── action_fakes.rs            moves, mutes, plugin commands, 3D sound and the pre-processor
├── connection_query_fakes.rs  the current server, the server, client and channel lists, statuses
├── fake_answers.rs            reading the plugin's arguments, writing answers, the SDK error codes
├── fake_table.rs              the table: a stub per slot, the fakes over theirs, `FAKED_FUNCTIONS`
├── memory_and_log_fakes.rs    `freeMemory`, errors, the log, the chat tab, paths, wave playback
├── mod.rs                     the module lines and the public re-exports
├── variable_fakes.rs          client, own-client, channel and server variables
└── tests/                     slot coverage, SDK order, and every fake called through the table
```

## How it works

`fake_table.rs` fills all 230 slots with `unexpected_slot::<N>` stubs (one per index, so a call
logs `HostCall::UnexpectedFunction { slot: N }` and answers `ERROR_not_implemented`), then writes
the 32 typed fakes over their slots from the one `faked_functions!` list, which also yields
`FAKED_FUNCTIONS`. Each fake has the exact signature of its slot, copies its arguments through
`fake_answers.rs`, takes the fake-state lock once, logs a `HostCall` and answers as TeamSpeak
does: `ERROR_not_connected` for an unknown server, `ERROR_client_invalid_id` or
`ERROR_channel_invalid_id` for an unknown ID, `ERROR_parameter_invalid` for a flag or setting it
does not model. A null vector argument (the listener's forward and up, which the plugins pass as
null) is logged as `None`. Strings and lists go out `malloc`ed and recorded in the ledger; `freeMemory`
takes them back.

The 32 faked functions: `getErrorMessage`, `freeMemory`, `logMessage`, `playWaveFile`,
`getPreProcessorConfigValue`, `setPreProcessorConfigValue`, `systemset3DListenerAttributes`,
`systemset3DSettings`, `channelset3DAttributes`, `requestClientMove`, `requestMuteClients`,
`requestUnmuteClients`, `getClientID`, `getClientSelfVariableAsInt`,
`setClientSelfVariableAsInt`, `setClientSelfVariableAsString`, `flushClientSelfUpdates`,
`getClientVariableAsInt`, `getClientVariableAsString`, `getClientList`, `getChannelOfClient`,
`getChannelVariableAsString`, `getChannelList`, `getChannelClientList`,
`getServerConnectionHandlerList`, `getServerVariableAsString`, `getConnectionStatus`,
`getConfigPath`, `getPluginPath`, `getCurrentServerConnectionHandlerID`,
`printMessageToCurrentTab`, `sendPluginCommand`.

## Boundaries

- Depends on: `fake_state` (the state, the ledger, the lock), `host_call`; `ts3plugin-sys` (the
  table, enums and error codes), `tfar_ts3_host` (the newtypes), `libc` through the ledger.
- Used by: `mock_host`, which installs the table into each plugin it starts.
- Rules: the faked set equals the set of `ts3Functions.*` calls in the original plugins under
  `reference/` (`every_function_the_tfar_plugins_call_has_a_typed_fake`); each faked slot index
  matches `crates/teamspeak/tfar_ts3_host/sdk_headers/ts3_functions.h`
  (`every_faked_slot_index_matches_the_sdk_header_order`); every other slot holds its own stub
  (`every_faked_slot_holds_its_fake_and_every_other_slot_its_own_stub`).
