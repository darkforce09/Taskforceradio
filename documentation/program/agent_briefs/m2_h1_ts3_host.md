**Status:** live

# M2 · H1 · `tfar_ts3_host`: the TeamSpeak plugin layer

You are agent H1, the TeamSpeak FFI author for the TFAR Rust port. Read
`/documentation/program/agent_briefs/shared_brief.md` first, then `/documentation/standards/coding_standards.md`
(the FFI and audio-thread rules).

## Background
- Both plugins run inside the Flatpak TeamSpeak 3.6.2 client (Linux x86_64, plugin API 26).
- The bindings come from `ts3plugin-sys` 0.5.0. Read its source in `~/.cargo/registry/src/*/ts3plugin-sys-0.5.0/` once it is fetched.
- The export layer is ours. The high-level `ts3plugin` crate is rejected (see `/documentation/program/research/library_survey.md`).
- `panic = "abort"`, so nothing on these paths may panic. TeamSpeak calls the voice callbacks on audio threads; everything else arrives on the TS main thread.
- What each plugin needs from TeamSpeak is listed in the behaviour specs, §4 "TS3 API usage" / "TeamSpeak API surface" (`/documentation/program/behaviour_specs/tfar_0_9_plugin.md`, `tfar_1_0_plugin.md`).

## Deliverables (crates/teamspeak/tfar_ts3_host; unsafe allow-listed, each `unsafe` with `// SAFETY:`)
1. **Layout verification.**
   - Fetch the official SDK headers for API 26 from GitHub `teamspeak/ts3client-pluginsdk`: `include/ts3_functions.h`, `include/plugin_definitions.h`, `teamspeak/public_definitions.h`, `teamspeak/public_rare_definitions.h` and `teamlog/logtypes.h`, at the commit/tag matching API 26 (client 3.5/3.6).
   - Store them under `crates/teamspeak/tfar_ts3_host/sdk_headers/` with a `README.md` saying where they came from.
   - Write a test that checks the `Ts3Functions` struct from `ts3plugin-sys`: `size_of` and the `offset_of` of every function pointer TFAR uses, against offsets computed from the header order (pointer size × index). The test parses the header in Rust at test time; no C compiler.
   - A mismatch fails the test. If `ts3plugin-sys` does not match, report it, and add the missing or misordered fields through our own `#[repr(C)]` mirror. Only do that if needed.
2. **`Ts3` handle:** a safe wrapper over the copied `Ts3Functions` table, stored once in `ts3plugin_setFunctionPointers` in a `OnceLock`.
   - One safe method per function TFAR uses. Each returns `Result<_, Ts3Error>`, where `Ts3Error` carries the TS error code and text via `getErrorMessage`.
   - C strings from TS are copied into Rust `String`s and freed with `freeMemory`.
   - Lists (client, channel) are read up to their 0 terminator, then freed.
   - Every method TFAR calls, from both specs §4. Where the versions differ, include the union.
3. **Export macro `ts3_plugin!`.** It takes an implementation of a `trait Ts3Plugin` and expands to every `#[unsafe(no_mangle)] extern "C" fn ts3plugin_*` the two plugins need:
   - required: name, version, apiVersion (= 26), author, description, setFunctionPointers, init, shutdown;
   - optional metadata: offersConfigure (0), registerPluginID, commandKeyword, processCommand, currentServerConnectionChanged, infoTitle, infoData, freeMemory, requestAutoload (1);
   - events: onConnectStatusChangeEvent, onNewChannelEvent, onNewChannelCreatedEvent, onDelChannelEvent, onChannelMoveEvent, onUpdateChannelEvent, onUpdateChannelEditedEvent, onUpdateClientEvent, onClientMoveEvent, onClientMoveTimeoutEvent, onClientMoveMovedEvent, onClientKickFromChannelEvent, onClientKickFromServerEvent, onClientBanFromServerEvent, onServerErrorEvent, onServerPermissionErrorEvent, onClientSelfVariableUpdateEvent, onPluginCommandEvent (API 26 signature, with invokerClientID, invokedByMe and invokerUniqueIdentity);
   - audio: onEditPostProcessVoiceDataEvent, onEditCapturedVoiceDataEvent, onEditMixedPlaybackVoiceDataEvent, onCustom3dRolloffCalculationClientEvent.

   Rules:
   - Every trait method has a default no-op, so each plugin implements only what it uses.
   - `name`/`version`/`author`/`description`/`infoTitle`/`commandKeyword` return `'static` NUL-terminated strings, from `&'static CStr` associated consts.
   - `infoData` allocates with `malloc`-compatible memory, so TS's `freeMemory` call back into the plugin can free it.
   - Audio callbacks hand the trait safe slices: `&mut [i16]`, channel count, sample count, and a `&mut u32` channel fill mask where present. Bounds come from the TS arguments, with an empty slice on null.
   - String arguments become `&str` (lossy); IDs become newtypes `ServerConnectionHandlerId(u64)`, `ClientId(u16)`, `ChannelId(u64)`.
4. **Logging.** A `tracing` subscriber layer that forwards events to TS `logMessage` with the channel name = plugin name, mapping the levels.
   - Never call TS from an audio thread: events raised on audio threads go through a bounded queue (`crossbeam-channel` or `rtrb`), drained by a small logger thread that starts in init and is joined in shutdown.
   - `install_logging(plugin_name)` returns a guard.
   - Also `print_to_current_tab(text)`, for the MessageBox replacement.
5. **Paths.** `plugin_path()` (via `getPluginPath(…, pluginID)`, the 3-argument API ≥ 21 form) and `config_path()`, both as `PathBuf` (F-018: no `%APPDATA%`).

## Tests
- The layout test.
- String-copy and free helpers, against a fake table built in the test.
- The `ts3_plugin!` macro expanded on a dummy plugin inside the test module: call the generated externs through function pointers with fake TS functions (an `extern "C"` fake table) and assert the trait sees the right arguments, including the audio-slice bounds and null handling.
- `infoData` allocation and free.
- The logging queue never calling TS on the emitting thread.

## Owned files
`crates/teamspeak/tfar_ts3_host/**`. External dependencies come from `[workspace.dependencies]`; report anything
missing (e.g. `libc` for `malloc`/`free`).

## Checks (each alone; `CARGO_TARGET_DIR=target-m2-h1`, `CARGO_BUILD_JOBS=4`; logs `target/agent-logs/h1-*.log`)
`cargo fmt -p tfar_ts3_host`; `cargo clippy -p tfar_ts3_host --all-targets -- -D warnings`; `cargo test -p tfar_ts3_host`;
`cargo xtask verify all` (report foreign violations only).

## Report additions
Layout test result (any mismatches); the `Ts3` method list; the trait method list.

Budget: L (350k tokens). Stop there and report what is done and what is not.
