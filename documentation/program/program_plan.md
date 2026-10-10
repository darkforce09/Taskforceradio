**Status:** live

# TFAR Rust port program plan

The plan of the program that rewrites both Task Force Arrowhead Radio versions (0.9.12 and 1.0) in
Rust, so Arma 3 under Proton talks to the native Linux TeamSpeak 3. This file holds what was found,
the binding decisions, how the work runs and the agent roster per milestone. The crates are in the
[crate catalogue](/documentation/program/crate_catalogue.md), the gates in
[laws and gates](/documentation/program/laws_and_gates.md), the bug fixes in
[bug fixes](/documentation/program/bug_fixes.md), the milestones in the
[roadmap](/documentation/roadmap.md), and live state in
[progress](/documentation/program/progress.md).

## 1. Context

- **Setup:** Arma 3 runs as the Windows build under Proton (compatdata 107410). TeamSpeak is the
  native Linux Flatpak `com.teamspeak.TeamSpeak3` 3.6.2 (plugin API 26); its sandbox shares the
  network and IPC with the host.
- **Why nothing connects:** both TFAR versions link game and TeamSpeak through Win32 named objects
  that live inside wineserver, where a native TeamSpeak plugin cannot reach them, and the installed
  plugins are Windows DLLs.
- **Goal:** rewrite both Arma extensions and both TeamSpeak plugins in Rust, from scratch, one of
  each per mod:

  | Mod | Extension | Plugin |
  |---|---|---|
  | 1.0 | `tfar_extension_1_0` | `tfar_plugin_1_0` |
  | 0.9.12 | `tfar_extension_0_9` | `tfar_plugin_0_9` |

  Shared logic lives in library crates. The plugins are native `.so` files; the extensions are
  Windows x64 DLLs; they talk over localhost TCP.
- **Untouched:** no SQF changes, and the extension name stays `task_force_radio_pipe`. The two mod
  trees sit whole under `reference/`, read-only; all Rust is new code that uses them as reference.

## 2. Verified findings

Each finding was checked against the reference trees; the behaviour specs in
[behaviour_specs](/documentation/program/behaviour_specs/README.md) hold the full evidence.

1. **0.9.12 transport is a named pipe the DLL creates.** `CreateNamedPipe` on
   `\\.\pipe\task_force_radio_pipe` (`reference/tfar_0_9_12/arma3/@task_force_radio/task_force_radio_pipe/task_force_radio_pipe.cpp:40`);
   the plugin opens it as a client (`reference/tfar_0_9_12/ts/src/plugin.cpp:507`) and every call is
   a blocking `TransactNamedPipe` round trip (`reference/tfar_0_9_12/arma3/@task_force_radio/task_force_radio_pipe/task_force_radio_pipe.cpp:53`).
2. **1.0 transport is shared memory the plugin creates.** `CreateFileMapping` of `Local\TFARSHAMEM`
   (`reference/tfar_1_0/ts/src/SharedMemoryHandler.cpp:271`), opened by the extension
   (`reference/tfar_1_0/extensions/task_force_radio_pipe/SharedMemoryTransfer.cpp:192`). Both kinds
   of object live in wineserver, invisible to a native `.so`.
3. **1.0 splits calls into sync and `~` async, and the extension answers async calls itself.**
   `DFRAME~` → `NEEDCFG`/`OK`, `MISSIONEND~` → empty reply and game tick 0, other async → `OK`
   (`reference/tfar_1_0/extensions/task_force_radio_pipe/SharedMemoryTransfer.cpp:250`).
4. **The 1.0 async queue is read last-in, first-out**
   (`reference/tfar_1_0/ts/src/SharedMemoryHandler.cpp:14`).
5. **1.0 MISSIONEND disconnects only through the zeroed game tick**; the plugin's own handler is a
   no-op (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:415`).
6. **Every SQF call uses the plain-string `callExtension` form**, TAB-separated, which `arma-rs`
   cannot serve; the export is `RVExtension(char*, int, const char*)`
   (`reference/tfar_1_0/extensions/task_force_radio_pipe/task_force_radio_pipe.cpp:9`).
7. **Clunk's HRTF filter is commented out in both trees**, so it adds only an interaural delay
   (`reference/tfar_1_0/ts/include/clunk/hrtf.cpp:216`, `reference/tfar_0_9_12/ts/include/clunk/hrtf.cpp:216`).
8. **1.0 direction and distance come from X3DAudio**, which is Windows-only
   (`reference/tfar_1_0/ts/src/helpers.cpp:30`).
9. **`Vector3D::normalized()` multiplies by `fast_invsqrt(length())`**, so directions have length
   about √distance (`reference/tfar_1_0/ts/src/datatypes.cpp:105`).
10. **1.0 matches radio subtypes by string length** (`reference/tfar_1_0/ts/src/plugin.cpp:116`).
11. **0.9 KILLED never runs**: the parser wants more than 2 tokens and Arma sends 2
    (`reference/tfar_0_9_12/ts/src/plugin.cpp:1775`); RELEASE_ALL_TANGENTS and TRACK fall through to
    `FAIL` (`reference/tfar_0_9_12/ts/src/plugin.cpp:1785`, `reference/tfar_0_9_12/ts/src/plugin.cpp:1799`).
12. **0.9 `inGame` turns true only when a serious channel name is set**
    (`reference/tfar_0_9_12/ts/src/plugin.cpp:2062`).
13. **Gain multiplication wraps instead of saturating in both versions**
    (`reference/tfar_0_9_12/ts/src/plugin.cpp:605`, `reference/tfar_1_0/ts/src/SampleBuffer.cpp:36`).
14. **The post-process callback reads and writes a second channel even for mono input in both
    versions** (`reference/tfar_0_9_12/ts/src/plugin.cpp:2831`, `reference/tfar_1_0/ts/src/plugin.cpp:768`).
15. **1.0 refuses to start beside an old TFAR module** through `GetModuleHandleA`
    (`reference/tfar_1_0/ts/src/plugin.cpp:301`); two Rust plugins need the dormant-plugin rule
    instead.
16. **Plugin-to-plugin commands are filtered by the sender's plugin name**: 0.9 checks the
    `task_force_radio` prefix (`reference/tfar_0_9_12/ts/src/plugin.cpp:3116`), 1.0 checks `TFAR`.
    TeamSpeak derives a plugin's name from its file name, so the installed file name decides who can
    talk to whom (open finding OF-002 in [progress](/documentation/program/progress.md)).
17. **The Windows-only surface is fully catalogued** (pipes, shared memory, WinInet, MessageBox,
    `getenv("appdata")` at `reference/tfar_1_0/ts/src/plugin.cpp:326`, `GetForegroundWindow` at
    `reference/tfar_1_0/ts/src/Teamspeak.cpp:207`); see
    [windows dependencies](/documentation/program/behaviour_specs/windows_dependencies.md).
18. **The sound files are real 48 kHz stereo WAVs in both trees**
    (`reference/tfar_1_0/ts/radio-sounds/on.wav`, `reference/tfar_0_9_12/ts/radio-sounds/on.wav`),
    so dist copies them as they are. The images and models in `reference/tfar_0_9_12` are Git LFS
    pointer stubs and are not used.

## 3. Binding decisions (owner, 2026-10-02)

| Id | Decision |
|---|---|
| D1 | Rewrite everything in Rust: extensions, plugins and all DSP. |
| D2 | Proton only. Extensions target `x86_64-pc-windows-gnu`; plugins target Linux `x86_64` only. |
| D3 | Fix all bugs (see [bug fixes](/documentation/program/bug_fixes.md)). Sound may differ from Windows players. Wire formats stay compatible. |
| D4 | Keep every feature: update check, Piwik `TRACK`, debug window, Tracy, API profiler, `/tfar` commands, `collectDebugInfo`. |
| D5 | The repository root is the Taskforceradio folder. The mods live in `reference/tfar_1_0/` and `reference/tfar_0_9_12/`, read-only. |
| D6 | Prefix `tfar_`, snake_case; the folder name equals the package name. Version tags are `_1_0` and `_0_9`. |
| D7 | crates/<category>/<crate>/ for every product crate, the 4 deliverables in their version category (no apps/ folder since 2026-10-10), tools/ for tooling. |
| D8 | Conventions follow TBD-Reforger and rerun: workspace package, dependencies and lints, tier metadata, crate anatomy, `cargo xtask verify` gates, cargo-deny, typos. Milestones live in the roadmap (TBD-subtitles style); program records follow the Reforger restructure style. |
| D9 | C/C++ oracles only in `tfar_golden_vectors`, built with `cc`: DSPFilters, SimpleComp and clunk straight from `reference/`, plus a vendored FAudio `F3DAudio.c` (zlib) in its oracle folder. |
| D10 | Port order: foundations, then the 0.9 track, then the 1.0 track. |
| D11 | Local git, Conventional Commits straight to `main`, explicit paths, a Co-Authored-By trailer, no remote yet. |
| D12 | `panic = "abort"` in dev and release. Panic-prone lints are denied, so production code has no panic path. |
| D13 | `cargo xtask dist` builds into the dist folder under target (tfar_1_0 and tfar_0_9). No script copies anything into a mod folder or into TeamSpeak; the owner copies by hand. |
| D14 | Execution: Opus sub-agents per slice, briefs and prompts under this folder, the orchestrator reviews and gates, progress.md holds the handoff. |
| D15 | `reference/tfar_1_0` is a clean upstream clone (upstream master @ `fce977fa`) with its own `.git`; the outer repository gitignores it and only the reference manifest guards it, excluding its .git folder. `reference/tfar_0_9_12` has no other version control and is committed as plain files (22 MB). |
| D16 | Licence decided later. M0 sets `license = "UNLICENSED"`; cargo-deny ignores private crates. Upstream is APL-SA; IRCAM HRIR data is credited beside the asset. |
| D17 | Toolchain pinned to 1.99.0, the newest stable on 2026-09-28. |
| D18 | Use libraries, don't reinvent the wheel. Product code is pure Rust; C/C++ appears only as a test oracle (D9). |
| D19 | HRTF uses the `hrtf` crate (real binaural HRTF) in both versions, fixing clunk's disabled KEMAR filter. The IRCAM HRIR sphere ships under assets with credit. |
| D20 | X3DAudio is replaced by `hrtf` for direction plus a small Rust gain model on `glam` for the distance curve and emitter cone. No full X3DAudio port, no double panning. FAudio's F3DAudio is only an oracle for the curve and cone maths. |
| D21 | Filters: `sci-rs` designs Butterworth LP/BP as second-order sections, `biquad` provides RBJ HP/LP coefficients, and everything processes in f64 Direct Form II Transposed. |
| D22 | `ts3plugin-sys` 0.5 (API 26) provides the TeamSpeak bindings; the export layer is ours. `arma-rs` is rejected (it cannot take TFAR's plain-string calls), so the `RVExtension` exports are ours. |

## 4. Execution model

Each milestone runs the same loop, under one orchestrator session:

1. **Prompts.** The orchestrator writes the milestone's agent prompts into
   [agent_briefs](/documentation/program/agent_briefs/README.md), refining the proposed roster in
   section 5 against the state of the tree.
2. **Agents.** Opus sub-agents run their slices, in parallel where the roster marks ∥ and the owned
   files are disjoint. Each agent follows the
   [shared brief](/documentation/program/agent_briefs/shared_brief.md) and ends with its six-section
   report.
3. **Review and gate.** The orchestrator reads every report and diff, runs the fast stage gate for
   the slice, triages findings into [progress](/documentation/program/progress.md), and sends a fix
   list back when the slice is not green.
4. **Commit.** One commit per slice, Conventional Commits, explicit paths.
5. **Handoff.** At the milestone close the orchestrator runs the full `cargo xtask ci ci-local`,
   ticks the [roadmap](/documentation/roadmap.md), and writes the handoff in progress.md.

The fast stage gate per slice is fmt, clippy and tests of the touched crates plus the documentation
gates; the full gate set runs at the milestone close (both defined in
[laws and gates](/documentation/program/laws_and_gates.md#stage-gate-and-milestone-gate)).

## 5. Milestones and agents

Budgets: S = 150k, M = 250k, L = 350k tokens. "∥" marks agents that run in parallel; "then" is a
hard order. M0 is the real roster; M1–M8 are proposals, refined at each milestone start.

| Milestone | Agents and content | Extra gates, checkpoints |
|---|---|---|
| M0 Workspace skeleton | **S0** (orchestrator): git init, tree moves verified byte-identical, toolchain and tools installed, specs and briefs saved. Then **A1** (M) standards, templates, CLAUDE.md, decisions, glossary ∥ **A2** (M) program records, roadmap, architecture, runbooks. Then **A3** (M) root config and all 38 crate skeletons. Then **A4** (L, amendment A1) the xtask tool: every gate, `ci ci-local`, `dist`, `new-crate`, `reference-manifest`. Close: full `ci-local`, Windows cross-build, commits per slice, handoff. | Perturbation proofs (inline test, 0.9→1.0 edge, reference byte); `reference-untouched`; dist placeholders with final names |
| M1 Foundations | **B1** (M) `tfar_geometry` and `tfar_arma_values` ∥ **B2** (S) `tfar_sample_buffer` and the shared tracing setup. | Tests pin F-004, F-006, F-009, F-010 |
| M2 TeamSpeak host | **H1** (L) `tfar_ts3_host` on `ts3plugin-sys`, with `size_of`/`offset_of` layout tests against the official API 26 headers. Then **H2** (M) `tfar_ts3_mock` ∥ **H3** (S) `tfar_ts3_settings_db`. Then **H4** (S) metadata exports in both plugin stubs. | **OC-ts:** the owner sees both stubs load in the Flatpak TeamSpeak; plugin file names settled (OF-002) |
| M3 Game link | **L1** (M) `tfar_game_link_protocol` and `tfar_game_link_server` ∥ **L2** (M) `tfar_arma_extension_abi` and `tfar_game_link_client`. Then **L3** (M) both extension apps, the stub plugins' listeners and `tfar_extension_probe`. | Windows cross-build; **OC-arma:** probe `PONG` under Proton's wine, then `TS_INFO\tPING` in Arma (BattlEye off) for both mods |
| M4 Audio primitives | **G1** (L) `tfar_golden_vectors`: cc-built DSPFilters, SimpleComp, clunk and F3DAudio oracles and fixtures. Then **P1** (M) `tfar_dsp_filters` ∥ **P2** (S) `tfar_dynamics` ∥ **P3** (L) `tfar_hrtf` with the IRCAM sphere ∥ **P4** (M) `tfar_spatial_audio` ∥ **P5** (S) `tfar_playback_mixer`. | Golden tests within tolerance; owner approves the HRIR download; hrtf click question settled |
| M5 Services and diagnostics | **W1** (M) `tfar_web_requests` against a local test server ∥ **W2** (M) `tfar_debug_window` on X11. | `ureq` and `egui` firewalls hold; window opens from the Flatpak TeamSpeak |
| M6 0.9 track | **V1** (M) `tfar_game_commands_0_9` and `tfar_plugin_commands_0_9` ∥ **V2** (M) `tfar_radio_effects_0_9`. Then **V3** (L) `tfar_client_state_0_9` and `tfar_radio_routing_0_9`. Then **V4** (L) `tfar_voice_pipeline_0_9`. Then **V5** (L) full `tfar_plugin_0_9` and `tfar_extension_0_9`. Then **V6** (M) mock scenario suites. | **OC-play-0.9:** live two-client session (SW, LR, DD, speakers, serious mode) |
| M7 1.0 track | **U1** (M) `tfar_plugin_settings_1_0` and `tfar_antennas_1_0` ∥ **U2** (M) `tfar_game_commands_1_0` and `tfar_plugin_commands_1_0` ∥ **U3** (M) `tfar_radio_effects_1_0`. Then **U4** (L) `tfar_client_state_1_0`. Then **U5** (M) `tfar_radio_routing_1_0`. Then **U6** (L) `tfar_voice_pipeline_1_0`. Then **U7** (L) full `tfar_plugin_1_0` and `tfar_extension_1_0`. Then **U8** (M) mock suites. | **OC-play-1.0:** as M6 plus antennas, intercom, SETCFG/NEEDCFG, focus-aware channel move |
| M8 Release readiness | **R1** (M) final `dist` with `radio-sounds` and the finished install runbook ∥ **R2** (M) bug-fix closure audit: every entry in bug_fixes.md names its test. | **OC-install:** the owner installs from the dist folder by hand and plays both mods |

## 6. Shared brief essentials

The orchestrator keeps these in the [shared brief](/documentation/program/agent_briefs/shared_brief.md)
before each launch.

- **Git:** agents never commit, stage, stash, reset or check out; the orchestrator commits.
- **Files:** each agent edits only the paths its prompt owns, and reports anything else as a finding.
- **Reference:** `reference/` is read-only, verified by `reference-untouched`.
- **Code:** compile at every save point; tests are never weakened; `cargo fmt` on owned crates.
- **Proof:** every new check gets a perturbation proof: break it, show it fails, restore.
- **Commands:** one command per shell call, each logged to its own file in the agent-logs folder
  under target; nothing over 60 s without a reason; no chained gate scripts.
- **Deferrals:** none silent; whatever cannot be done is named with the reason.
- **Reports:** at most 250 words in six sections: files; commands, exit codes, logs; tests and gates;
  perturbation proofs; findings; not done or deviations.

## 7. Risks

| Risk | Mitigation |
|---|---|
| BattlEye blocks unsigned extensions | Documented: BattlEye off, or servers without BattlEye |
| TS SDK API 26 struct-layout mismatch crashes TeamSpeak | Bindings checked against the official headers; mock plus `size_of`/`offset_of` tests in M2 |
| `cargo-deny`/`typos` not installed | Installed in S0 |
| `reference/tfar_1_0` is not in the outer repository (D15) | The reference manifest detects drift; its upstream `.git` is the backup; the reference README records how to re-clone `fce977fa` |
| Windows players run the original plugin | Plugin-command and metadata formats stay byte-compatible; only the local DSP differs; the plugin file name keeps the TeamSpeak plugin name (OF-002) |
| Stale Windows DLLs in the Flatpak plugins folder | The install runbook tells the owner to remove them by hand |
| `hrtf` clicks on fast-moving sources | Measured in M4 against the open question in the roadmap |
| `ts3plugin-sys` has low upstream activity | Layout tests pin it; the export layer is ours either way |
