**Status:** live

# M2 · H2 + H3 · `tfar_ts3_mock` and `tfar_ts3_settings_db`

You are agent H2, the TeamSpeak test-host author for the TFAR Rust port. You own both crates in this prompt. Read
`/documentation/program/agent_briefs/shared_brief.md` first (including the **Build directory rule**), then
`/documentation/standards/coding_standards.md`, and the `tfar_ts3_host` crate (READMEs + public API). Its
`ts3_plugin!` macro is what the plugins export.

## `tfar_ts3_mock` (tools/tfar_ts3_mock), lib + bin, unsafe allow-listed
A fake TeamSpeak client that loads a real plugin `.so` and drives it.
- **`MockHost::load(path) -> Result<MockHost>`** (libloading). It resolves every `ts3plugin_*` symbol (optional ones as `Option`), builds a full `ts3plugin_sys::Ts3Functions` table whose every slot is an `extern "C"` fake, and calls them in TeamSpeak's order:
  1. `setFunctionPointers`
  2. `registerPluginID("mock_plugin_id")`
  3. `init`
- **Fake state** sits in a process-global `parking_lot::Mutex<FakeTeamSpeak>`, so the extern fakes can reach it. It holds:
  - servers, channels and clients with nicknames, metadata and talking flags;
  - the current server handler;
  - preprocessor values (`vad`);
  - plugin and config paths (temp dirs).
- **Call log.** Every call is recorded as a typed `HostCall` enum: `SendPluginCommand { command, target }`, `RequestMuteClients(ids)`, `RequestUnmuteClients(ids)`, `SetClientSelfVariableAsString`, `SetClientSelfVariableAsInt`, `PlayWaveFile(path)`, `RequestClientMove`, `LogMessage`, `PrintMessageToCurrentTab`, `Systemset3DSettings`, `Channelset3DAttributes`, `SetPreProcessorConfigValue`, and so on. Strings are returned with TS ownership semantics: allocated with `libc::malloc`, freed by `freeMemory`. The mock checks that the plugin frees what it receives and reports leaks in `MockHost::leak_report()`.
- **Drivers:** `connect_server(...)`, `client_joins`, `client_moves`, `set_talking`, `deliver_plugin_command(from_client, plugin_name, command)`, and the audio hooks:
  - `post_process_voice(client, samples, channels)`
  - `captured_voice(samples, channels, edited)`
  - `mixed_playback(samples, channels, fill_mask)`
  - `custom_3d_rolloff(client) -> f32`

  Each returns the buffers after the plugin ran. There are also `info_data(id, kind)`, `process_command(text)` and `shutdown()`.
- **`locate_cdylib(crate_name) -> Option<PathBuf>`** finds `lib<name>.so` beside or above the running test executable (`target-*/debug/` and `deps/`). It lets plugin apps' integration tests load their own build.
- **Bin `tfar_ts3_mock <plugin.so>`:** loads the plugin, prints its metadata (name, version, apiVersion, author, description, autoload), runs init, connects a fake server, then shutdown, and prints the call log.
- **Tests:**
  - fake-table slot coverage (every function TFAR uses has a fake);
  - string allocation and free;
  - a self-built test plugin: put a tiny cdylib under `tools/tfar_ts3_mock/tests/fixture_plugin/` as its own workspace member? **No.** Instead, test the fakes directly through the table's function pointers. Loading real plugins is the apps' job, using `locate_cdylib`.
- The crate root needs the four-point header (`src/main.rs` and `src/lib.rs`).

## `tfar_ts3_settings_db` (crates/teamspeak/tfar_ts3_settings_db)
- `read_push_to_talk_delay(settings_db: &Path) -> Result<PushToTalkDelay>`, using `rusqlite` with `bundled` from the workspace dependencies.
  - Query: `SELECT value FROM Profiles WHERE key='Capture/Default/PreProcessing'` (`reference/tfar_1_0/ts/src/plugin.cpp:344`, `reference/tfar_0_9_12/ts/src/plugin.cpp:2248`).
  - Parse the value line by line: `delay_ptt=true` → enabled; `delay_ptt_msecs=<n>` → ms.
  - `PushToTalkDelay { enabled: bool, milliseconds: u32 }`, with `effective() -> Duration` (zero when not enabled).
- Open the database read-only (`SQLITE_OPEN_READ_ONLY`), because TeamSpeak has it open.
- A missing file, table or row → a typed error. The caller treats it as zero delay.
- **Tests** on temp databases:
  - enabled with ms;
  - disabled;
  - missing row;
  - garbage value;
  - CRLF lines.

## Owned files
`tools/tfar_ts3_mock/**`, `crates/teamspeak/tfar_ts3_settings_db/**`. If a workspace dependency or feature is missing,
make one Edit to the root `Cargo.toml` under the matching banner, re-reading the file first; report it.

## Checks (each alone, with your target dir; logs `target/agent-logs/h2-*.log`)
fmt, clippy `--all-targets -D warnings`, test for both crates; `cargo xtask verify all` (report foreign violations only).

## Report additions
Public API list; the fake-slot coverage list.

Budget: L (350k tokens). Stop there and report what is done and what is not.
