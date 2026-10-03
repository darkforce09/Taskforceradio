# Mock TeamSpeak host

The `tfar_ts3_mock` crate: a fake TeamSpeak host that loads a plugin .so and drives it through
scripted scenarios. The plugin apps' integration tests and developers run it.

## Contents

```text
tools/tfar_ts3_mock/
├── Cargo.toml  the `tfar_ts3_mock` package (lib + bin): tier 2, workspace lints and dependencies
└── src/        the host, the fake function table, the fake state, the plugin loader and the binary
```

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
  `process_command`, `shutdown`, `calls`, `take_calls`, `leak_report`, `with_fake_teamspeak`),
  `HostOptions`, `ServerSetup`, `InfoKind`, `CommandOutcome`, `VoiceBlockOutcome`,
  `CapturedVoiceOutcome`, `MOCK_PLUGIN_ID`, `HOSTED_API_VERSION`.
- `PluginLibrary`, `PluginMetadata`, `PluginExports` and its function pointer types.
- `HostCall`, `HostQuery`, `SentCommandTarget`; `FakeTeamSpeak`, `FakeServer`, `FakeChannel`,
  `FakeClient`; `LeakReport`, `UnfreedAllocation`.
- `locate_cdylib`, `FAKED_FUNCTIONS`, `FakedFunction`, `SLOT_COUNT`, `mock_error_text`.
- `Error`, `Result`, `CRATE_ROLE`, `prelude` (which also re-exports the `tfar_ts3_host` newtypes).
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
  plugin. Tier 2 in `tools`: every workspace dependency has a lower tier and follows the edge
  matrix in `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin and the TeamSpeak API it uses.
- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin and the TeamSpeak API it uses.
- [Coding standards](/documentation/standards/coding_standards.md) — the FFI and test rules the
  mock holds.
