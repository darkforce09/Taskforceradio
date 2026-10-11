# TeamSpeak host layer

The `tfar_ts3_host` crate: the safe layer over the TeamSpeak 3 plugin SDK (API 26): exports,
callbacks, function table, logging. Used by `tfar_plugin_0_9`, `tfar_plugin_1_0`.

## How it works

A plugin app implements `Ts3Plugin` on a `static` and expands `ts3_plugin!` over it; the macro
generates every `ts3plugin_*` symbol both TFAR plugins need, as `extern "C"` shells that hand the
raw arguments to the crate's glue, which converts them (newtype IDs, lossy `&str`, bounded sample
slices) and calls the trait. `ts3plugin_setFunctionPointers` copies TeamSpeak's `TS3Functions`
table (the `#[repr(C)]` struct of `ts3plugin-sys`) into a process-wide `Ts3` handle; its methods
are the only way the apps call TeamSpeak, and they copy and free every string and list TeamSpeak
allocates. `install_logging` routes `tracing` through a bounded queue to a logger thread that calls
`logMessage`, so an audio thread never calls TeamSpeak. The layout of the `ts3plugin-sys` table is
checked against the headers in `sdk_headers/`: all 230 slots match the SDK order at plugin API 26,
and the crate uses its own `#[repr(C)]` mirror nowhere.

TeamSpeak loads a plugin app, looks up the exports `ts3_plugin!` generated (`plugin_exports`) and
calls `ts3plugin_setFunctionPointers`, which installs the `Ts3` handle (`ts3_functions`). Every later
callback reaches the app's `Ts3Plugin` implementation with safe arguments: IDs from `host_ids`,
statuses from `host_values`, text as `&str`, voice blocks as bounded slices. The app calls back into
TeamSpeak through `Ts3`, whose methods return `Ts3Error` with TeamSpeak's code and text on failure,
and logs through `tracing`, which `logging` forwards to `logMessage` from its own thread. A plugin's
voice callbacks count their lock waits and skipped blocks in an `AudioDiagnostics`
(`audio_diagnostics`), lock-free, for its service thread to log.

Threads: TeamSpeak's main thread runs the lifecycle and event exports and may call `Ts3`; its audio
threads run the voice exports, which only touch the samples they are given; the logger thread is
the only other thread this crate starts.

`tests/fake_ts3_table.rs` builds a `Ts3Functions` whose every slot aborts unless a test fills it;
the unit tests of `ts3_functions` and `plugin_exports` share it. `tests/sdk_layout.rs` parses the
headers in `sdk_headers/` and checks the table offsets and every SDK value the crate passes.

## Modules

### `sdk_headers/`

`src/tests/sdk_layout.rs` reads `ts3_functions.h` at test time, numbers every `(*name)(` function
pointer inside `struct TS3Functions` in order, and checks that `size_of::<Ts3Functions>()` is the
pointer count times the pointer size and that every function `tfar_ts3_host` calls sits at offset
index × pointer size in the `ts3plugin-sys` struct. The same test parses the enums of the other
headers and checks every property flag, status and target value the crate passes to TeamSpeak. No C
compiler is involved; the headers are read as text only.

- Encoding: C headers, ASCII, LF line ends, exactly as published upstream; the file names are the
  upstream names with the upstream folders flattened.
- Layout: upstream `include/ts3_functions.h` and `include/plugin_definitions.h`, then
  `include/teamspeak/public_definitions.h` and `include/teamspeak/public_rare_definitions.h`, then
  `include/teamlog/logtypes.h`. The `#include` lines inside still name the upstream folders; nothing
  resolves them.
- Adding a file: copy it unchanged from the same upstream commit, add a line above and to the
  checksum list below, and run `cargo test -p tfar_ts3_host`.

- Producers: TeamSpeak Systems GmbH, repository
  <https://github.com/teamspeak/ts3client-pluginsdk>, tag `26`, commit
  `b2f4b8e574d13c584ba8cae0c5ab6e1a0c3048e6` (2023-07-31, "update readme"); between that tag and
  `master` only the upstream README and CI file differ. Fetched on 2026-10-02. Upstream states
  "Copyright TeamSpeak Systems GmbH. All rights reserved."; the copies serve the private build's tests
  and ship in no artifact.
- Consumers: the layout tests in `crates/teamspeak/tfar_ts3_host/src/tests/sdk_layout.rs`.

SHA-256 of the copies (equal to the upstream files at that commit):

```text
7630586c5f3dfd0ab37f31b3358633d24784f9257c1d1c04defb440a673fc91c  logtypes.h
09839fd1aa076df3e12a2f1d01b0c54fa00d129df6786ded4d3221f3ea7f8326  plugin_definitions.h
b41171b42e0bc525f77cbcc01d37b7e0449642fd177851564473ddd033b7f870  public_definitions.h
a201480d4ce253cca0dd471be9e495fe0a4673b302d2897a2dd9d172dac606c8  public_rare_definitions.h
0cad4df43e0dd2cf713a2db4fcb8ca84bb2fab97e1a54502f5e62f2e49bc9643  ts3_functions.h
```

### `src/logging/`

```text
any thread (audio included)          logger thread "tfar-teamspeak-log"
tracing::info!(…) ─▶ TeamSpeakLogLayer ─try_send─▶ bounded queue ─recv─▶ sink ─▶ Ts3::log_message
                     (format, never blocks;                              (channel = plugin name)
                      full → drop and count)
```

`install_logging(plugin_name)` (from `ts3plugin_init`, after the table is installed) creates one
process-wide queue on first use and makes a registry with `TeamSpeakLogLayer` the global `tracing`
subscriber; later installs reuse both, since `tracing` takes a global default only once. It then
starts the logger thread, whose sink calls `logMessage` with the plugin name as channel and `0` as
log ID. The layer maps `ERROR`, `WARN`, `INFO`, `DEBUG` and `TRACE` to `LogLevel_ERROR`, `WARNING`,
`INFO`, `DEBUG` and `DEVEL`, filters by the level given (`DEBUG` by default), and drops every event
while no logger thread runs. Dropping the `LoggingGuard` (in `ts3plugin_shutdown`) stops
forwarding, queues the stop message, lets the thread finish what was queued and joins it.

### `src/plugin_exports/`

A plugin app writes `static PLUGIN: MyPlugin = …;` and `tfar_ts3_host::ts3_plugin!(PLUGIN);` at its
crate root. The macro (split over `export_macro.rs`, `event_exports_macro.rs` and
`audio_exports_macro.rs`) expands to one `#[unsafe(no_mangle)] extern "C"` shell per export, with the
argument list of the SDK export prototypes at API 26 (the original keeps them in
`reference/tfar_1_0/ts/src/plugin.h`). Each shell bundles its raw arguments and calls a function
under `$crate::export_glue`, which `export_glue.rs` gathers from the three glue files:

```text
TeamSpeak ──dlsym──▶ ts3plugin_onEditPostProcessVoiceDataEvent (macro shell, app crate)
                       │ raw pointers and counts
                       ▼
                     audio_glue::post_process_voice_data
                       │ null and size checks → &mut [i16], &[u32], &mut u32
                       ▼
                     Ts3Plugin::on_edit_post_process_voice_data (the app's code)
```

The metadata strings are the trait's `&'static CStr` constants, so the pointers never dangle.
`ts3plugin_setFunctionPointers` installs the `Ts3` handle, `ts3plugin_registerPluginID` stores the
ID in it, and `ts3plugin_init` fails with `1` when no table is installed. `ts3plugin_infoData`
copies the plugin's text into a `malloc` block that TeamSpeak hands back to `ts3plugin_freeMemory`,
which calls `free`. Voice callbacks run on TeamSpeak's audio threads: their glue allocates nothing,
locks nothing and calls no TeamSpeak function, and runs each voice block under a `DenormalGuard`:
flush-to-zero and denormals-are-zero set in MXCSR on entry and the caller's mode restored on exit,
as the originals set the same mode, so decaying filter and HRTF tails never fall into slow
denormal arithmetic. A null or inconsistent buffer becomes an empty slice with `frames == 0`, and a
null mask, `edited` or `volume` pointer a local value.

### `src/ts3_functions/`

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

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_ts3_host             # the layout, function-table, export and logging tests
cargo test -p tfar_ts3_host sdk_layout  # only the checks against the SDK headers
```

## Configuration

None: the crate reads no setting. TeamSpeak's own paths come from `Ts3::plugin_path` (the raw
`getPluginPath`), `Ts3::plugin_folder` (`getPluginPath`, else the folder of the loaded plugin
library; what the apps use) and `Ts3::config_path`.

## Public surface

- `ts3_plugin!` and `Ts3Plugin`: the exports and the callbacks a plugin app implements.
- `Ts3`: one safe method per TeamSpeak function either TFAR plugin calls, each returning
  `Result<_, Ts3Error>` where TeamSpeak can fail.
- `install_logging`, `LoggingGuard`, `print_to_current_tab`: logging into TeamSpeak and the chat
  tab.
- The SDK newtypes and values: `ServerConnectionHandlerId`, `ClientId`, `ChannelId`,
  `ConnectStatus`, `ClientProperty` and the rest in [Modules](#modules).

## Boundaries

- Depends on: no workspace crate; `ts3plugin-sys` (the raw table and SDK enums), `libc` (`malloc`
  and `free` for `infoData`, `dladdr` for the plugin folder), `arc-swap` (the plugin ID),
  `tracing`, `tracing-subscriber`, `crossbeam-channel`, `thiserror`.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`.
- Rules: no panic or unwind crosses into TeamSpeak; every C string handed over stays alive for the
  call; the `ts3plugin-sys` layout matches the SDK headers (`src/tests/sdk_layout.rs`). Tier 0 in
  `crates/teamspeak`: every workspace dependency has a lower tier and follows the category edge matrix (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin and the TeamSpeak API it uses.
- [TFAR 1.0 plugin behaviour](/documentation/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin and the TeamSpeak API it uses.
- [Coding standards](/documentation/standards/code_and_documentation.md) — the FFI and audio-thread
  rules this crate holds.
