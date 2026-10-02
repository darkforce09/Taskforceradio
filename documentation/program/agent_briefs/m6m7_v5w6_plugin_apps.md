**Status:** live

# M6/M7 · V5 (0.9) / W6 (1.0) · The full TeamSpeak plugin apps

This brief serves two agents. **V5** owns `apps/tfar_plugin_0_9`. **W6** owns `apps/tfar_plugin_1_0`. Each works
only on its own app. Read `/documentation/program/agent_briefs/shared_brief.md` first (including the **Build directory
rule**), then:
- `/documentation/standards/coding_standards.md` (FFI and audio-thread rules)
- `/documentation/architecture/{game_link_protocol,audio_pipeline}.md`
- **your version's behaviour spec in full** (`/documentation/program/behaviour_specs/tfar_0_9_plugin.md` or `tfar_1_0_plugin.md`)
- `/documentation/program/bug_fixes.md`
- the current app crate (L3's stub: exports, lifecycle, listener, mock-host test)
- the README and public API of every workspace crate your manifest lists

Every domain decision already lives in a library crate. **The app is wiring, threads and TeamSpeak side effects.**
If you find logic missing from a library, write it in the app only if it is TS-specific. Otherwise report it as a
finding and implement a minimal app-private version.

## Wiring both apps need
1. **Plugin state.**
   - One `PluginRuntime` behind an `Arc`, created in `init`, holding:
     - `Ts3` (from `tfar_ts3_host`);
     - per server connection, `parking_lot::Mutex<ServerState>` with the version's client-state, the routing `MuteTracker`, and the voice processor with its scratch buffers;
     - the playback mixer and the WAV library (radio-sounds preloaded off the audio thread at init from `plugin_path()/radio-sounds/`);
     - the HRIR library, loaded once from `plugin_path()/tfar_hrir/IRC_1002_C.bin`, the dist layout. A missing sphere means HRTF is off: direction falls back to the version's simple ILD, with a TS chat message saying the file is missing;
     - the game-link server;
     - the settings (1.0);
     - the service thread handle;
     - the dormant flag.
   - Lock order is documented, and no TS call is made while holding a server lock in an audio callback. Collect decisions under the lock, then call TS after releasing it.
2. **Game link handler.** It replaces the stub.
   - Sync requests are parsed with the version's game-commands crate, applied to the state, and answered per the reply tables.
   - 1.0 async messages go through the server's FIFO worker into the same apply path.
   - Side effects run through a small `TeamSpeakActions` trait (mute, unmute, rename, metadata, plugin command, play sound, client move, 3D neutralising, VAD and input deactivation), so tests use the mock.
3. **Service thread** (per spec): every 100 ms, the expiry sweep and mute refresh every 4 s, metadata every 4 s, game-timeout and connect/disconnect transitions, and the serious-channel move (1.0: every 4 s while in game, gated by FOCUS frames unless `moveWhileTabbedOut`).
4. **Dormant rule** (game link protocol §Dormant). Until the first game activity: no metadata, nickname, mute, channel or plugin commands, and audio passes through.
5. **TeamSpeak events.**
   - connect status: on ESTABLISHED, `systemset3DSettings(1,1)`, reset data, the nickname list;
   - client move, kick, timeout;
   - channel events (1.0 serious-channel cache);
   - update client;
   - `onPluginCommandEvent`, using the version's plugin-commands crate with its sender filter;
   - `onClientSelfVariableUpdateEvent` (talking → VOLUME broadcast);
   - `onCustom3dRolloff` → 1.0;
   - `infoData` from the version's metadata helper;
   - `onServerErrorEvent` returns 1 when a return code is present.
6. **Audio callbacks.**
   - post-process: copy to stereo, run `process_sender`, write back;
   - captured: own voice for speakers, append to the mixer stream;
   - mixed playback: `mixer.mix_into`.
   - None of them allocate. All of them `try_lock` with a pass-through fallback when the state is busy, and document it.
7. **PTT and VAD** (spec): press and release handling, the delayed release thread using `tfar_ts3_settings_db::read_push_to_talk_delay` (read once at init; 1.0 also adds `tangentReleaseDelay`), VAD save and restore, `CLIENT_INPUT_DEACTIVATED`, the broadcast, and the local start/end sounds by subtype folder.
8. **Lifecycle sounds:** `radio-sounds/on`/`off` via `playWaveFile`; remote start/end via the mixer with gain and position.
9. **Services:**
   - `tfar_web_requests::spawn_update_check` at init; the message goes to the TS tab.
   - `TRACK` → `build_track_request_*` + `spawn_track`.
10. **Shutdown:** game end if in game, metadata disabled then cleared, unmute all, stop the link server, join the service, release and debug threads, and drop logging last. All of it within 2 s.

## V5 specifics (0.9)
- Metadata: the 0.9 format.
- Plugin name `task_force_radio` (dist file name).
- `commandKeyword` is empty.
- The F-022 in-game rule; the F-024 original nickname; `isOtherRadioPluginEnabled` channels.

## W6 specifics (1.0)
- **Settings:** SETCFG is applied through `tfar_plugin_settings_1_0`, plus the needcfg push to `GameLinkServer::set_needcfg`, and serious-mode re-check on every SETCFG.
- **Antennas:** `RadioTwrAdd` / `RadioTwrDel` update the antenna manager.
- **Commands:**
  - `/tfar` (`processCommand`): `diag`, `pos`, `full…`, `debug` (only with `allowDebugging`), `rstflt`.
  - `debug` opens `tfar_debug_window` and publishes snapshots from the service thread every 500 ms while it is open.
  - `collectDebugInfo` writes a text dump under `config_path()/logs/<dd-mm_HH-MM-SS>/`: the client list, settings and recent game messages, using a bounded ring buffer (OF-008).
- **Versions:** the minimum-plugin-version warning is a one-time TS message. The nickname-too-short warning likewise.
- **Features:** `tracy` and `api_profiler` cargo features forward to `profiling` / `tracy-client`, and are off by default.

## Tests (integration tests through `tfar_ts3_mock`, plus unit tests on app-private modules)
- **Scenario scripts:**
  1. load;
  2. connect a fake server with 3 clients;
  3. drive game messages over a real game-link client (`TFAR_GAME_LINK_PORT`) — VERSION / SETCFG, FREQ, POS ×3, TANGENT press;
  4. assert the plugin commands sent, the metadata text, the rename, the mute calls (serious mode), and the PTT input deactivation sequence;
  5. deliver a remote TANGENT plugin command and push voice through `post_process_voice` for an in-range SW sender: the output differs from the input and is band-limited, while a non-TFAR sender passes through;
  6. the mixed-playback sound after the remote press;
  7. shutdown with no leaks (`leak_report`).
- **Dormant plugin:** no calls before game activity.
- **1.0:** MISSIONEND disconnect and the off sound; needcfg; antennas; the debug window not opening without a display, with a TS message instead.

## Owned files
V5: `apps/tfar_plugin_0_9/**`. W6: `apps/tfar_plugin_1_0/**`. One root-manifest Edit at most (re-read first), reported. Never touch
the other app or the library crates. Report missing library functions as findings, with the exact API you needed.

## Checks (each alone, with your target dir; logs `target/agent-logs/<v5|w6>-*.log`)
fmt, clippy `--all-targets -D warnings`, test for your app; `cargo xtask dist` with your target dir (confirm your `.so`, and
`nm -D` shows the `ts3plugin_*` exports); `cargo xtask verify all` (report foreign violations only).

## Report additions
A behaviour-coverage table (spec section → code module → test); the lock order; the findings for library gaps.

Budget: L (350k tokens). Stop there and report what is done and what is not.
