# Mock TeamSpeak host

The `tfar_ts3_mock` crate: a fake TeamSpeak host that loads a plugin .so and drives it through
scripted scenarios. The plugin apps' integration tests and developers run it.

## How it works

`MockHost::load` opens a plugin `.so` with `libloading`, resolves its `ts3plugin_*` exports and
starts it as TeamSpeak does: `setFunctionPointers` with a fake `Ts3Functions` table, then
`registerPluginID("mock_plugin_id")`, then `init`. The table holds a typed `extern "C"` fake for
each of the 32 functions the two TFAR plugins call and a recording stub in each of the other
198 slots. The fakes answer from one process-global `FakeTeamSpeak` (servers, channels, clients,
pre-processor values, plugin and config folders), log every call as a `HostCall`, and hand out
strings and lists with `malloc`, which the plugin returns through `freeMemory`; `leak_report()`
lists what it did not. The drivers (`connect_server`, `client_joins`, `talk_status_change`,
`deliver_plugin_command`, `post_process_voice` and the rest) change the fake state and raise the callbacks TeamSpeak would.
One host at a time owns the fake state, so parallel tests take turns. A plugin app's integration
test finds its own build with `locate_cdylib("tfar_plugin_1_0")`.

```text
test ──▶ MockHost driver ──▶ fake state (locked, changed, unlocked)
                         └─▶ ts3plugin_on…Event (plugin code)
                                   └─▶ fake Ts3Functions slot ──▶ fake state + call log
```

`mock_host` opens a plugin through `plugin_library`, begins a session in `fake_state` (waiting
for any other host, resetting the state, creating the folders) and hands the plugin the table
from `fake_functions`. From then on two kinds of code touch the fake state: the drivers on the
test's thread, which change it and then call a plugin export with the lock released, and the
fakes on whichever thread the plugin calls from, which read or change it, append a `HostCall`
and, for strings and lists, record a `malloc`ed block in the ledger. The lock is re-entrant over
a `RefCell`, so a nested use fails at once instead of deadlocking.

## Modules

### `src/fake_functions/`

`fake_table.rs` fills all 230 slots with `unexpected_slot::<N>` stubs (one per index, so a call
logs `HostCall::UnexpectedFunction { slot: N }` and answers `ERROR_not_implemented`), then writes
the 32 typed fakes over their slots from the one `faked_functions!` list, which also yields
`FAKED_FUNCTIONS`. Each fake has the exact signature of its slot, copies its arguments through
`fake_answers.rs`, takes the fake-state lock once, logs a `HostCall` and answers as TeamSpeak
does: `ERROR_not_connected` for an unknown server, `ERROR_client_invalid_id` or
`ERROR_channel_invalid_id` for an unknown ID, `ERROR_parameter_invalid` for a flag or setting it
does not model; `getPluginPath` answers the plugin folder only for the registered plugin ID and an
empty path for any other, as TeamSpeak answers a stale ID after a reload. A null vector argument
(the listener's forward and up, which the plugins pass as
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

### `src/fake_state/`

`host_session.rs` keeps one `FakeTeamSpeak` in a `static` (a function pointer carries no
context, so the fakes reach it globally) behind a re-entrant `parking_lot` mutex over a
`RefCell`. `HostSession::begin` waits on a condition variable until no other session is active,
then replaces the state with a fresh one whose plugin and config paths are new temporary folders
(or the ones given) and whose registered plugin ID is `mock_plugin_id` (a reload replaces it);
its drop frees every block still handed out and wakes the next waiter. The
ledger in `allocation_ledger.rs` maps each outstanding block's address to the function that
handed it out; a free of an unknown or already freed address is recorded, never passed to
`free`.

### `src/mock_host/`

`MockHost::start` refuses a plugin not built for API 26, begins the session, then calls
`setFunctionPointers`, `registerPluginID("mock_plugin_id")` and `init`; a non-zero `init` is
`Error::InitFailed`. An event driver changes the fake state under the lock, releases it and calls
the export, so the plugin's queries see the change; a callback the plugin does not export is
skipped. `set_talking` only sets `CLIENT_FLAG_TALKING`; `talk_status_change` sets it and then
raises `onTalkStatusChangeEvent`, as TeamSpeak does when a client starts or stops talking. An
audio driver copies the caller's samples, passes the counts, one speaker bit per channel and the
flags TeamSpeak passes, and returns what the plugin left; a missing export is
`Error::MissingExport`. `reload(plugin_id)` plays TeamSpeak's "Reload" on the loaded library:
`shutdown` (unless it ran), `setFunctionPointers`, `registerPluginID(plugin_id)` and `init`; the
fake state then answers `getPluginPath` for the new ID only, so a plugin that kept the old ID gets
the empty path TeamSpeak gives it. `shutdown` runs once per start or reload, from the call or the
drop, before the library unloads.

`tests/in_process_plugin.rs` defines `extern "C"` exports inside the test binary and hands them
to the host over the test executable itself, so the tests run the real start order, drivers and
memory rules without a separately built plugin.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_ts3_mock                                  # the fake table, fakes and drivers
cargo run -p tfar_ts3_mock -- target/debug/libtfar_plugin_1_0.so  # metadata, init, connect, shutdown, calls
```

The binary exits `0` when the plugin starts, connects and shuts down with a clean leak report,
`1` when a step fails or memory leaks, and `2` for a wrong command line.

## Configuration

None: the crate reads no setting. `HostOptions` replaces the temporary plugin and config folders
the fakes answer with; the default pre-processor value is `vad=false`.

## Public surface

- `MockHost` (`load`, `load_with_options`, `start`, the event and audio drivers, `info_data`,
  `process_command`, `reload`, `registered_plugin_id`, `shutdown`, `calls`, `take_calls`,
  `leak_report`, `with_fake_teamspeak`),
  `HostOptions`, `ServerSetup`, `InfoKind`, `CommandOutcome`, `VoiceBlockOutcome`,
  `CapturedVoiceOutcome`, `MOCK_PLUGIN_ID`, `HOSTED_API_VERSION`.
- `PluginLibrary`, `PluginMetadata`, `PluginExports` and its function pointer types.
- `HostCall`, `HostQuery`, `SentCommandTarget`; `FakeTeamSpeak`, `FakeServer`, `FakeChannel`,
  `FakeClient`; `LeakReport`, `UnfreedAllocation`.
- `locate_cdylib`, `FAKED_FUNCTIONS`, `FakedFunction`, `SLOT_COUNT`, `mock_error_text`.
- `Error`, `Result`, `prelude` (which also re-exports the `tfar_ts3_host` newtypes).
- The binary `tfar_ts3_mock <plugin.so>`.

## Boundaries

- Depends on: `tfar_ts3_host` (the SDK newtypes and values), `tfar_game_link_protocol` and
  `tfar_game_link_client` (for scenarios that play the game side); `ts3plugin-sys` (the raw table,
  enums and error codes), `libloading`, `libc`, `parking_lot`, `thiserror`; a plugin `.so` built
  for plugin API 26.
- Used by: the plugin apps' integration tests (`tfar_plugin_0_9`, `tfar_plugin_1_0`) and
  developers at the command line.
- Rules: every function the original plugins call has a typed fake, in the SDK slot order
  (`src/fake_functions/tests/fake_table.rs`); every slot holds a valid function; no plugin code
  runs under the fake-state lock; a scenario's outcome depends only on the scenario and the
  plugin. Tier 2 in `tools`: every workspace dependency has a lower tier and follows the category edge matrix (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin and the TeamSpeak API it uses.
- [TFAR 1.0 plugin behaviour](/documentation/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin and the TeamSpeak API it uses.
- [Coding standards](/documentation/standards/code_and_documentation.md) — the FFI and test rules the
  mock holds.
