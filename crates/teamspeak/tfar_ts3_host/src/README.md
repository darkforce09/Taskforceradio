# TeamSpeak host layer source

The source of `tfar_ts3_host`: the SDK identifiers and values, the safe function-table handle, the
export layer and the logging bridge, each in its own module.

## Contents

```text
crates/teamspeak/tfar_ts3_host/src/
├── crate_role.rs    `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs         `Ts3Error` for a TeamSpeak call, `Error` and `Result` for the rest
├── host_ids.rs      the newtypes `ServerConnectionHandlerId`, `ClientId`, `ChannelId`
├── host_values.rs   connect status, visibility, log level, property flags, targets, `Ts3Vector`
├── lib.rs           the crate root: module header, `mod` lines and re-exports
├── logging/         the `tracing` layer, its queue and the logger thread that calls `logMessage`
├── plugin_exports/  `Ts3Plugin`, `ts3_plugin!` and the glue the exports call
├── prelude.rs       the public names for glob import, without `Error` and `Result`
├── tests/           the SDK layout test, the fake function table, the crate-role test
└── ts3_functions/   `Ts3`: one safe method per TeamSpeak function TFAR calls
```

## How it works

TeamSpeak loads a plugin app, looks up the exports `ts3_plugin!` generated (`plugin_exports`) and
calls `ts3plugin_setFunctionPointers`, which installs the `Ts3` handle (`ts3_functions`). Every later
callback reaches the app's `Ts3Plugin` implementation with safe arguments: IDs from `host_ids`,
statuses from `host_values`, text as `&str`, voice blocks as bounded slices. The app calls back into
TeamSpeak through `Ts3`, whose methods return `Ts3Error` with TeamSpeak's code and text on failure,
and logs through `tracing`, which `logging` forwards to `logMessage` from its own thread.

Threads: TeamSpeak's main thread runs the lifecycle and event exports and may call `Ts3`; its audio
threads run the voice exports, which only touch the samples they are given; the logger thread is
the only other thread this crate starts.

`tests/fake_ts3_table.rs` builds a `Ts3Functions` whose every slot aborts unless a test fills it;
the unit tests of `ts3_functions` and `plugin_exports` share it. `tests/sdk_layout.rs` parses the
headers in `sdk_headers/` and checks the table offsets and every SDK value the crate passes.

## Public surface

- `Ts3`: the function-table handle; used by both plugin apps.
- `Ts3Plugin`, `ts3_plugin!` and the argument and answer types (`ClientMove`, `Invoker`,
  `PluginCommand`, `ServerError`, `ServerPermissionError`, `PostProcessVoiceData`,
  `MixedPlaybackVoiceData`, `CapturedVoiceData`, `InitStatus`, `CommandHandling`, `ErrorDisplay`,
  `InfoItem`); used by both plugin apps.
- `ServerConnectionHandlerId`, `ClientId`, `ChannelId`, `ConnectStatus`, `Visibility`, `TalkStatus`,
  `LogLevel`, `ClientProperty`, `ChannelProperty`, `ServerProperty`, `PluginCommandTarget`,
  `Ts3Vector`.
- `install_logging`, `install_logging_at`, `LoggingGuard`, `TeamSpeakLogLayer`,
  `print_to_current_tab`.
- `Ts3Error`, `Error`, `Result`, `CRATE_ROLE`, `prelude`; `export_glue` is public only for the
  macro expansion and hidden from the docs.

## Boundaries

- Depends on: `ts3plugin-sys`, `libc`, `arc-swap`, `tracing`, `tracing-subscriber`,
  `crossbeam-channel`, `thiserror`; the TeamSpeak 3 client (plugin API 26) at run time.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`.
- Rules: `lib.rs` holds only the header, attributes, `mod` lines and re-exports
  (`cargo xtask verify crate-anatomy`); every `unsafe` block has a `// SAFETY:` comment naming the
  memory owner (`cargo xtask verify unsafe-allowlist`); tests live in `tests/` folders
  (`cargo xtask verify test-placement`).
