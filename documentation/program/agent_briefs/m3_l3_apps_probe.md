**Status:** live

# M3 · L3 · Both extension apps, plugin stubs with the game link, the probe

You are agent L3, the apps author for M2/M3 of the TFAR Rust port. Read
`/documentation/program/agent_briefs/shared_brief.md` first (including the **Build directory rule**). Then read
`/documentation/architecture/game_link_protocol.md` (authoritative), `/documentation/program/behaviour_specs/extensions.md`,
and the public APIs of `tfar_arma_extension_abi`, `tfar_game_link_client`, `tfar_game_link_server`,
`tfar_game_link_protocol` and `tfar_ts3_host`.

## `crates/version_0_9/tfar_extension_0_9` and `crates/version_1_0/tfar_extension_1_0` (final versions)
- **Wiring:** `arma_extension!` with a handler over a process-wide `GameLinkClient` (`OnceLock`).
  - Flavour per app. Port from `tfar_game_link_protocol::port(flavour, debug)`, with `debug =
    arma_command_line_contains("-tfdebug")`.
  - `RVExtensionVersion` returns `"tfar_extension_0_9 <crate version>"` / `"tfar_extension_1_0 <crate version>"`.
- **Strings:** map `SyncOutcome` and connection states to the game-facing strings **exactly** per the protocol spec's per-flavour sections and "Game-facing strings" table.
  - 0.9: `Pipe not opened from TS plugin`, `Pipe was closed by TS`, `Not connected to TeamSpeak`.
  - 1.0: `~` async (`DFRAME` → `NEEDCFG`/`OK`, `MISSIONEND` → empty, others `OK`); no live session → `Not connected to TeamSpeak`; sync timeout → empty.
  - Empty input → empty output.
- **Focus (1.0):** call `foreground_window_is_arma()` at most every 250 ms, on calls, and send `FOCUS` on change.
- **Logic layout:** put it in a testable module, e.g. `src/call_handling.rs`, as a pure function over a small trait wrapping the client, so the string mapping is unit-tested with a fake. Add an integration test that runs the real client against an in-test `GameLinkServer` on an ephemeral port (`ServerConfig.port = 0`).
- **The unsafe reason must be a `//` comment on the line directly above `#![allow(unsafe_code)]`**: the gate rejects `reason =` inside the attribute.

## `crates/version_0_9/tfar_plugin_0_9` and `crates/version_1_0/tfar_plugin_1_0` (stubs that M6/M7 complete)
- **Exports:** implement `tfar_ts3_host::Ts3Plugin` and export it with `ts3_plugin!`. Exact original metadata:
  - **0.9:** name `Task Force Arma 3 Radio`, version `0.9.12`, author `[TF]Nkey`, description `Radio Addon for Arma 3`, info title `Task Force Radio Status (0.9.12)`, command keyword `""`.
  - **1.0:** name `Task Force Arrowhead Radio`, version `1.0.0.1`, author `[TF]Nkey and Dedmen`, description `Radio Addon for Arma 3`, info title `Task Force Radio Status`, command keyword `tfar`.
  - Both: apiVersion 26, autoload 1, no configure dialog.
- **Lifecycle:**
  - `init`: install logging (`tfar_ts3_host` logging guard) and start a `GameLinkServer` on the flavour's port. The debug port is used when the environment variable `TFAR_GAME_LINK_DEBUG=1` is set (roadmap open question; this default is recorded as an amendment).
  - The stub handler answers `TS_INFO\tPING` → `PONG`. Other sync requests get the version's unknown reply (0.9 `FAIL`, 1.0 `UNKNOWN COMMAND`); async messages are ignored.
  - `shutdown`: stop the server and drop the logging guard; everything is joined.
- **Integration test per plugin**, through `tfar_ts3_mock` as a dev-dependency:
  1. `locate_cdylib`, `MockHost::load`;
  2. assert the metadata;
  3. connect with a raw game-link client (or `tfar_game_link_client`) to the plugin's port. Use a test-only env var `TFAR_GAME_LINK_PORT` override, documented, so tests don't fight over fixed ports;
  4. expect `PONG`;
  5. shutdown cleanly.

  If `crate-tiers` rejects an app → tool dev-dependency, report it with the gate output instead of working around it.
  H2 is writing `tfar_ts3_mock` in parallel. If it isn't ready, write your test against its brief's API (`/documentation/program/agent_briefs/m2_h2_mock_settings_db.md`) and run it once the crate builds.

## `tools/tfar_extension_probe` (Windows exe)
- `tfar_extension_probe <path-to-dll> [call ...]` loads the DLL with `libloading` (allowed by the firewall for this tool), resolves `RVExtension` and `RVExtensionVersion`, prints the version, then performs each call (default calls: `TS_INFO\tPING`, `TS_INFO\tVERSION`) with a 10240-byte output buffer, the size Arma uses, printing the replies.
- It must build for `x86_64-pc-windows-gnu`; on Linux it builds too (loading a `.so` for host testing).
- Unsafe glue: the four-point header, plus the `// SAFETY:` rule, plus the allow-list. The probe isn't on the unsafe allow-list yet. Report it as needed, so the orchestrator or C0 adds it to `tools/xtask/src/layout.rs`. Don't edit xtask.

## Owned files
`apps/**`, `tools/tfar_extension_probe/**`. One Edit to the root `Cargo.toml` is allowed if a dependency entry is missing
(re-read it first); report it.

## Checks (each alone, with your target dir; logs `target/agent-logs/l3-*.log`)
fmt; clippy `--all-targets -D warnings` for the 4 apps and the probe; the windows clippy line from CLAUDE.md §4 plus
`-p tfar_extension_probe`; `cargo build --target x86_64-pc-windows-gnu -p tfar_extension_0_9 -p tfar_extension_1_0 -p tfar_extension_probe`;
the tests of the 4 apps; `cargo xtask dist` (confirm the 6 artifacts); `cargo xtask verify all` (report foreign violations only).

## Report additions
A string-mapping test table (input situation → output string, test name); the dist artifact list.

Budget: L (350k tokens). Stop there and report what is done and what is not.
