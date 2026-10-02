# Mock TeamSpeak host source

The source of `tfar_ts3_mock`: the host and its drivers, the fake function table, the fake
TeamSpeak state, the plugin loader, the call log and the binary.

## Contents

```text
tools/tfar_ts3_mock/src/
├── crate_role.rs        `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs             `Error`, the `thiserror` enum of every failure, and `Result`
├── fake_functions/      the typed `extern "C"` fakes and the `Ts3Functions` table that holds them
├── fake_state/          `FakeTeamSpeak`, the allocation ledger and the one-host-at-a-time session
├── host_call.rs         `HostCall`, `HostQuery`, `SentCommandTarget`: the call log's records
├── lib.rs               the crate root: module header, `mod` lines and re-exports
├── locate_cdylib.rs     `locate_cdylib`: a plugin app's own `.so` from inside its test executable
├── main.rs              the binary: load, metadata, init, connect, shutdown, call log
├── mock_host/           `MockHost`: start and shutdown, the event drivers and the audio drivers
├── plugin_library/      `PluginLibrary`: the loaded `.so`, its resolved exports and its metadata
├── prelude.rs           the public names and the `tfar_ts3_host` newtypes for glob import
├── temporary_folder.rs  the session's temporary plugin and config folders
└── tests/               unit tests of `CRATE_ROLE` and `locate_cdylib`
```

## How it works

`mock_host` opens a plugin through `plugin_library`, begins a session in `fake_state` (waiting
for any other host, resetting the state, creating the folders) and hands the plugin the table
from `fake_functions`. From then on two kinds of code touch the fake state: the drivers on the
test's thread, which change it and then call a plugin export with the lock released, and the
fakes on whichever thread the plugin calls from, which read or change it, append a `HostCall`
and, for strings and lists, record a `malloc`ed block in the ledger. The lock is re-entrant over
a `RefCell`, so a nested use fails at once instead of deadlocking.

## Public surface

- `MockHost` and its option, setup and outcome types; `PluginLibrary`, `PluginMetadata`,
  `PluginExports`; `HostCall` and its parts; the fake state types; `LeakReport`.
- `locate_cdylib`, `FAKED_FUNCTIONS`, `SLOT_COUNT`, `mock_error_text`, `Error`, `Result`,
  `CRATE_ROLE`, `prelude`.
- The binary's `main`, which developers run.

## Boundaries

- Depends on: `tfar_ts3_host`, `ts3plugin-sys`, `libloading`, `libc`, `parking_lot`, `thiserror`.
- Used by: the plugin apps' integration tests and the binary.
- Rules: `lib.rs` holds only the header, attributes, `mod` lines and re-exports
  (`cargo xtask verify crate-anatomy`); every `unsafe` block has a `// SAFETY:` comment naming who
  owns the memory (`cargo xtask verify unsafe-allowlist`); tests live in `tests/` folders
  (`cargo xtask verify test-placement`).
