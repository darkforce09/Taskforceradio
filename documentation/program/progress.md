**Status:** live

# Program progress

The single progress tracker of the TFAR Rust port. The orchestrator updates it after every agent
report and gate run, and commits it with each slice, so a lost session costs at most one slice. A
fresh session reads Current state and Handoff first, then the
[program plan](/documentation/program/program_plan.md).

## Current state

| Field | Value |
|---|---|
| Branch | `main` |
| Current milestone | M8 — closed by C1 in code (M0–M8 code-complete); only the owner's live checks remain |
| Last green commit | 450c626 — the first commit on which `cargo xtask ci ci-local` holds every step (21 of 21, run by C1 on 0ced2f5 plus the C1 changes, 2026-10-03); the orchestrator fills the hash |
| Next action | Commit C1 and fill its hash above; then the owner's live checks OC-ts, OC-arma and OC-install from the [install runbook](/documentation/runbooks/manual_install_and_test.md) |
| Blocked on | nothing |

Row format: `- [ ] <ID> (<budget>) <role> — status — commit — notes`. A status is `pending`,
`running`, `done`, `blocked` or `fix-list`. Owner checkpoints are rows of their own. Budgets: S =
150k, M = 250k, L = 350k tokens.

## Milestones

### M0 Workspace skeleton

- [x] S0 (orchestrator) git init, tree moves, toolchain and tools, specs and briefs — done — 7bc0e9c — see the execution log
- [x] A1 (M) standards, templates, CLAUDE.md and AGENTS.md, decisions, glossary, reference README — done — 65c6752 — ran in parallel with A2
- [x] A2 (M) program records, roadmap, architecture, runbooks — done — 8dd2e53 — open findings OF-001 to OF-011
- [x] A3 (M) root config and all 38 crate skeletons — done — fbef38e — started in parallel with A1 (amendment A2)
- [x] A4 (L) xtask gates, `ci ci-local`, `dist`, `new-crate`, `reference-manifest` — done — a233f60 — budget per amendment A1; see OF-012
- [x] C0 (M) gate cleanup and the M0 close records — done — 4fdaaf5 — added by amendment A6; [brief](/documentation/program/agent_briefs/m0_c0_gate_cleanup.md)
- [x] deny and typos config (orchestrator) — done — 8c87196 — closes OF-014 and OF-015
- [x] R1 (M) records and tooling pass: `dist` target folder and HRIR copy, bug fixes, library map, edges, protocol and audio docs — done — 1998d0f + 119517b + c4add2d — [brief](/documentation/program/agent_briefs/m4_r1_records_tooling.md); amendments R1-1, R1-2
- [x] Close (orchestrator) full `ci-local`, Windows cross-build, commits per slice, handoff, memory note — done — 450c626 — folded into C1 (M8 wave 3)

### M1 Foundations

- [x] B1 (M) `tfar_geometry` and `tfar_arma_values` — done — 1b30920 — ran early (amendment A3)
- [x] B2 (S) `tfar_sample_buffer` and the tracing rule — done — 1a24f01 — ran early (amendment A3)

### M2 TeamSpeak host

- [x] H1 (L) `tfar_ts3_host` on `ts3plugin-sys`, layout tests against the API 26 headers — done — 8b84a9c — SDK headers kept in `crates/teamspeak/tfar_ts3_host/sdk_headers/`
- [x] H2 (L) `tfar_ts3_mock` and `tfar_ts3_settings_db` (H3 folded in) — done — fb6621d — [brief](/documentation/program/agent_briefs/m2_h2_mock_settings_db.md)
- [x] H4 (S) metadata exports in both plugin stubs — done — 52bda9e — done inside L3 (amendment A7)
- [ ] OC-ts (owner) both stubs load in the Flatpak TeamSpeak — pending — — live check deferred by the owner

### M3 Game link

- [x] L1 (M) `tfar_game_link_protocol` and `tfar_game_link_server` — done — 97df8ce — ran before M2 completed (amendment A4)
- [x] L2 (M) `tfar_arma_extension_abi` and `tfar_game_link_client` — done — 97df8ce — committed with L1
- [x] L3 (L) both extension apps, the stub plugins with the game link, `tfar_extension_probe` — done — 52bda9e — [brief](/documentation/program/agent_briefs/m3_l3_apps_probe.md)
- [ ] OC-arma (owner) probe `PONG` under Proton's wine, then `TS_INFO\tPING` in Arma — pending — — live check deferred by the owner

### M4 Audio primitives

- [x] P1 (M) `tfar_dsp_filters` and `tfar_dynamics` (P2 folded in) — done — 7901de3 — [brief](/documentation/program/agent_briefs/m4_p1_p2_filters_dynamics.md)
- [x] P3 (L) `tfar_hrtf` with the IRCAM sphere — done — 2a74b9b — HRIR download approved by the owner; [brief](/documentation/program/agent_briefs/m4_p3_hrtf.md)
- [x] P4 (M) `tfar_spatial_audio` and `tfar_playback_mixer` (P5 folded in) — done — 2049b4a — [brief](/documentation/program/agent_briefs/m4_p4_p5_spatial_playback.md)
- [x] G1 (L) `tfar_golden_vectors` and the golden tests against the crates — done — 2be5b03 — after P1–P5 (amendment A5); results below; [brief](/documentation/program/agent_briefs/m4_g1_golden_vectors.md)

G1 results (live oracles, FAudio 6839b88e, tag 26.10):

| Comparison | Worst result |
|---|---|
| Butterworth and RBJ filters against DSPFilters | ≤ 6e-8 relative RMS error |
| Compressor against SimpleComp | 1.1e-16 |
| Distance curve and emitter cone against F3DAudio | ≤ 2.2e-7 |
| Stereo `i16` filter path against `processFilterStereo` | 1 LSB, clipped samples included (after the R1-2 clip fix) |

### M5 Services and diagnostics

- [x] S1 (M) `tfar_web_requests` and `tfar_debug_window` (S2 folded in) — done — 420d37e — adds `url`, `winit` and `fastrand` to the workspace; [brief](/documentation/program/agent_briefs/m5_s1_s2_services_debug_window.md)

### M6 0.9 track

- [x] V1 (M) `tfar_game_commands_0_9` and `tfar_plugin_commands_0_9` — done — f534c75 + 61f66f6 — 61f66f6 adds F-027 and F-028; [brief](/documentation/program/agent_briefs/m6_v1_commands_0_9.md)
- [x] E1 (L) `tfar_radio_effects_0_9` and `tfar_radio_effects_1_0` — done — ff99769 — [brief](/documentation/program/agent_briefs/m6m7_e1_radio_effects.md)
- [x] V3 (L) `tfar_client_state_0_9` and `tfar_radio_routing_0_9` — done — 4ffce7d — F-032, F-033; [brief](/documentation/program/agent_briefs/m6_v3_state_routing_0_9.md)
- [x] V4 (L) `tfar_voice_pipeline_0_9` — done — 4018568 — F-051, the +4 dB makeup (D23); [brief](/documentation/program/agent_briefs/m6m7_v4w4_voice_pipelines.md)
- [x] V5 (L) `tfar_plugin_0_9`, the full 0.9 plugin — done — a742c88 — F-071; [brief](/documentation/program/agent_briefs/m6m7_v5w6_plugin_apps.md)

### M7 1.0 track

- [x] W1 (M) `tfar_plugin_settings_1_0`, `tfar_game_commands_1_0` and `tfar_plugin_commands_1_0` — done — 654db3b — [brief](/documentation/program/agent_briefs/m7_w1_commands_settings_1_0.md)
- [x] W3 (L) `tfar_client_state_1_0`, `tfar_radio_routing_1_0` and `tfar_antennas_1_0` — done — 5dfd3f5 — F-041 to F-047; [brief](/documentation/program/agent_briefs/m7_w3_state_routing_antennas_1_0.md)
- [x] W4 (L) `tfar_voice_pipeline_1_0` — done — f8e9d5f — F-061 to F-063, the 1.0 attenuation `10^(−1.5·d/maxD)`; [brief](/documentation/program/agent_briefs/m6m7_v4w4_voice_pipelines.md)
- [x] W6 (L) `tfar_plugin_1_0`, the full 1.0 plugin — done — fd9593a — F-081, F-082; [brief](/documentation/program/agent_briefs/m6m7_v5w6_plugin_apps.md)

### M8 Completion waves

All code is written. The completion plan replaces the plan's proposed M8 roster (R1 and R2 there):
three waves close the records, fill the library gaps the plugin apps worked around privately, move
the apps onto the new library APIs and close every milestone. A wave starts when the one before it
is committed.

Wave 1 (parallel):

- [x] R3 (M) records pass after the plugin apps: bug fixes, edges, audio and game link docs, D23–D26, this file — done — acccf42 — amendment R3-1; [brief](/documentation/program/agent_briefs/m8_r3_records.md)
- [x] L4 (L) infrastructure gaps: two ports and `queue_async` on the game link server, the talk-status export, `Option` listener vectors, the mock driver, cancellable web requests, the non-blocking mixer stream — done — b67d9a9 — [brief](/documentation/program/agent_briefs/m8_l4_infrastructure_gaps.md)
- [x] P5 (L) audio-path gaps: allocation-free `is_over_radio_into`, F-091, `apply_local_tangent`, `render_remote_tangent_sound` — done — b7f2efd — F-091, OF-027 closed; [brief](/documentation/program/agent_briefs/m8_p5_audio_path_gaps.md)

Wave 2 (parallel, after wave 1):

- [x] V6 (M) `tfar_plugin_0_9` onto the new APIs, D24, D25, the HRTF reset on talk status, the shutdown proof — done — 110de0e — OF-028 closed; [brief](/documentation/program/agent_briefs/m8_v6w7_app_migration.md)
- [x] W7 (M) `tfar_plugin_1_0` onto the new APIs, D24, D25, the HRTF reset on talk status, the shutdown proof — done — 0ced2f5 — OF-028 closed; [brief](/documentation/program/agent_briefs/m8_v6w7_app_migration.md)

Wave 3 (after wave 2):

- [x] C1 (M) full `cargo xtask ci ci-local`, `cargo xtask dist` and its artefacts, roadmap ticks, the install runbook, the bug-fix closure, the first green commit — done — 450c626 — amendments C1-1 to C1-7 (A12); the step and artefact tables are in the execution log; [brief](/documentation/program/agent_briefs/m8_c1_milestone_close.md)

Owner checkpoints, after C1:

- [ ] OC-install (owner) install from the dist folder by hand and play both mods — pending — — live check deferred by the owner

## Execution log

| Date | Event | Detail | Logs |
|---|---|---|---|
| 2026-10-02 | S0 tree move | `task-force-arma-3-radio` → `reference/tfar_1_0` and `task-force-arma-3-radio-0.9.12` → `reference/tfar_0_9_12` as plain renames; verified byte-identical: 789 files (1.0, excluding `.git`) and 919 files (0.9.12), sha256 lists before and after matched | session scratchpad: pre_1_0.sha, pre_0_9.sha, premove_counts.txt |
| 2026-10-02 | S0 toolchain | Rust 1.99.0 with targets `x86_64-unknown-linux-gnu` and `x86_64-pc-windows-gnu`, cargo-deny 0.20.2 and typos 1.50.3 installed | session scratchpad: toolchain.log, tools_install.log |
| 2026-10-02 | S0 records | Behaviour specs (0.9 plugin, 1.0 plugin, extensions, Windows dependencies), research (conventions, libraries) and the M0 agent briefs saved under this folder | — |
| 2026-10-02 | S0 Git LFS | `reference/tfar_0_9_12` holds Git LFS pointer stubs (models, images) committed as plain bytes. LFS is neutralised locally through `.git/info/attributes` with the line `reference/** -filter -diff -merge -text`, so no LFS filter ever runs on the reference trees. A fresh clone must add the same line (see the [resume runbook](/documentation/runbooks/continue_in_claude_code.md#troubleshooting)) | — |
| 2026-10-02 | S0 commit | `chore(workspace): s0 baseline with reference trees and program specs` — 7bc0e9c | — |
| 2026-10-02 | M0 launch | A1 and A2 launched in parallel from their briefs; A4's budget raised to L (amendment A1) | — |
| 2026-10-02 | A2 report | Roadmap, program records (plan, catalogue, laws and gates, bug fixes, progress, folder READMEs), architecture (layout, game link protocol, audio pipeline) and runbooks written; 26 bug-fix entries (6 both, 12 1.0-only, 8 0.9-only) | target/agent-logs (A2) |
| 2026-10-02 | A1, A2 commits | `docs(workspace): standards, templates, decisions and CLAUDE.md` — 65c6752; `docs(program): roadmap, program plan, catalogue, laws and runbooks` — 8dd2e53 | — |
| 2026-10-02 | A3 commit | `chore(workspace): root config and 37 crate skeletons` — fbef38e (xtask is the 38th crate, A4's) | target/agent-logs (A3) |
| 2026-10-02 | B1, B2 commits | `feat(tfar-sample-buffer): …` — 1a24f01; `feat(tfar-geometry): positions, directions and Arma value parsing` — 1b30920 | target-m1-b1, target-m1-b2 |
| 2026-10-02 | A4 commit | `build(xtask): verify gates, ci-local lane, dist, new-crate, manifest` — a233f60 | target-a4 |
| 2026-10-02 | L1, L2 commit | `feat(tfar-game-link): protocol, plugin server and extension client` — 97df8ce | target-m3-l1, target-m3-l2 |
| 2026-10-02 | Briefs | M1–M4 briefs — 78675fa; C0, H2, L3, P3 briefs and the build directory rule — 2396c91; S1, V1, W1 briefs — f6430a2 | — |
| 2026-10-02 | H1 commit | `feat(tfar-ts3-host): safe TeamSpeak layer over ts3plugin-sys` — 8b84a9c | target-m2-h1 |
| 2026-10-02 | Launch | C0, H2, L3, P1, P3 and P4 launched in parallel, each building into its own `target-<agent id>` folder | — |
| 2026-10-03 | C0 run | Gate debts cleared (language-bans, firewalls, link-check, readme-coverage, unsafe allow-list); `ci ci-local` stops at fmt on an L3 file, so each step was rerun alone | target/agent-logs/c0-*.log |
| 2026-10-03 | Commits | C0 — 4fdaaf5; deny and typos config — 8c87196; P1 — 7901de3; P3 — 2a74b9b; P4 — 2049b4a; V1 — f534c75, 61f66f6; S1 — 420d37e; W1 — 654db3b; H2 — fb6621d; L3 — 52bda9e; G1 — 2be5b03; E1 — ff99769 | target-<agent id> folders |
| 2026-10-03 | Briefs | G1 and E1 — 13c0ef5; V3 and W3 — aa546d0; R1 — 9b216f4; V4 and W4 — 3f8e398; V5 and W6 — b4af759 | — |
| 2026-10-03 | Launch | G1, E1 and R1 run in parallel, then V3 and W3; G1 and E1 reported and are committed | — |
| 2026-10-03 | R1 dist | `CARGO_TARGET_DIR=target-r1 cargo xtask dist` wrote both version folders under target-r1/dist/, each with its `.so`, the DLL, 18 radio-sounds files and tfar_hrir/ | target/agent-logs/r1-dist.log |
| 2026-10-03 | Commits | R1 — 1998d0f, 119517b, c4add2d; V3 — 4ffce7d; W3 — 5dfd3f5; V4 — 4018568; W4 — f8e9d5f; V5 — a742c88; W6 — fd9593a; handoff — 6bad8f2 | target-<agent id> folders |
| 2026-10-03 | Briefs | Wave 1 (R3, L4, P5) — d9606f0; waves 2 and 3 (V6, W7, C1) — 81ebf0d | — |
| 2026-10-03 | Launch | Wave 1: R3, L4 and P5 run in parallel, each in its own `target-<agent id>` folder | — |
| 2026-10-03 | Commits | R3 — acccf42; P5 — b7f2efd; L4 — b67d9a9; V6 — 110de0e; W7 — 0ced2f5 | target-<agent id> folders |
| 2026-10-03 | C1 ci-local | `CARGO_TARGET_DIR=target-c1 cargo xtask ci ci-local` on 0ced2f5 plus the C1 changes: all 21 steps held (fmt, clippy, clippy-windows, test with 1830 passed, 0 failed, 3 ignored doc examples in 114 suites, deny, typos, and the 15 verify gates) | target/agent-logs/c1-ci-local-1.log, target/agent-logs/c1-ci-run1/ |
| 2026-10-03 | C1 dist | `CARGO_TARGET_DIR=target-c1 cargo xtask dist` wrote target-c1/dist/: tfar_0_9 with `task_force_radio_linux_amd64.so` (6 065 280 B) and `task_force_radio_pipe_x64.dll` (964 608 B), tfar_1_0 with `TFAR_linux_amd64.so` (12 954 080 B) and `task_force_radio_pipe_x64.dll` (967 168 B), each with radio-sounds/ (18 files, identical across the versions) and tfar_hrir/ (`IRC_1002_C.bin` 772 656 B, `CREDITS.md` 2 167 B). Each `.so` exports the 40 `ts3plugin_*` functions; each DLL exports `RVExtension` and `RVExtensionVersion` and imports only Windows system DLLs (kernel32, user32, ntdll, msvcrt, ws2_32, userenv, bcryptprimitives, api-ms-win-core-synch), no libgcc or libwinpthread | target/agent-logs/c1-dist.log |
| 2026-10-10 | Layout | The four deliverables move from `apps/` into `crates/version_0_9/` and `crates/version_1_0/`; the gates key the deliverable rules by crate name ([decision](/documentation/decisions/workspace.md#2026-10-10--deliverables-live-in-their-version-category-no-apps-folder-d7)) | — |

## Amendments

- **A1 (2026-10-02): A4's budget is L (350k), not M.** The plan's M0 roster gave A4 an M budget; the
  gate set (16 repository gates, each with a perturbation test, plus `ci`, `dist`, `new-crate` and
  `reference-manifest`) needs L. The [A4 brief](/documentation/program/agent_briefs/m0_a4_xtask_gates.md)
  already says L.
- **A2 (2026-10-02): A3 started in parallel with A1**, not after A1's templates; A3 followed the plan's
  conventions directly.
- **A3 (2026-10-02): M1–M3 slices ran early, in parallel with the end of M0.** B1, B2, L1, L2 and H1
  ran before the M0 close; each built into its own target folder.
- **A4 (2026-10-02): L1 and L2 ran before M2 completed.** The game link crates depend on no
  TeamSpeak crate, so the plan's M2-then-M3 order did not bind them.
- **A5 (2026-10-02): G1 moves after P1–P5.** G1 compares live oracle output against the finished
  crates, so no fixtures pass between crates and P1–P5 need nothing from G1 first.
- **A6 (2026-10-02): the C0 cleanup agent is added to M0** to clear the gate debts the parallel slices
  left and to bring these records up to date. At launch: `tfar_extension_probe` joins the unsafe
  allow-list (it calls `RVExtension` through raw pointers), C0 leaves the files of the running agents
  and the root Cargo.toml alone, and C0 records the C0, H2, L3 and P3 briefs here.
- **A7 (2026-10-02): slices are folded.** H2 also does H3 (`tfar_ts3_settings_db`), P1 also P2
  (`tfar_dynamics`), P4 also P5 (`tfar_playback_mixer`), and L3 does H4 (the stubs' metadata exports).
- **A8 (2026-10-03): R1 leaves E1's and G1's files and the root Cargo.toml alone (R1-1).** G1 and E1
  ran in parallel with R1 and held uncommitted changes, the root manifest included.
- **A9 (2026-10-03): R1 also takes G1's follow-ups (R1-2):** `tfar_golden_vectors` joins the unsafe
  allow-list; the stale "fixtures" wording goes; `tfar_dsp_filters` writes `i16::MAX`/`i16::MIN`
  for a stereo `i16` sample beyond full scale, as `processFilterStereo` does, and the golden
  clipped-sample tolerance tightens to 1 LSB.
- **A10 (2026-10-03): R3 runs beside L4 and P5, which change code at the same time (R3-1).** R3 adds
  F-091 to the bug fixes if P5 lands it before R3 finishes; otherwise the next records pass does.
- **A11 (2026-10-03): the completion waves replace the plan's M8 roster.** The plan proposed R1 (dist
  and runbook) and R2 (bug-fix audit) for M8; the records, the library gaps, the app migration and
  the milestone close now run as waves 1–3 (R3, L4, P5; V6 ∥ W7; C1), and the M0 close folds into
  C1.
- **A12 (2026-10-03): C1 also clears the known leftovers (C1-1 to C1-7).** The P5 brief's link to
  the removed `remote_sounds.rs` becomes plain text; the audio pipeline document, the catalogue's
  and the shared brief's edge tables follow the code (the 1.0 plugin no longer depends on
  `tfar_dsp_filters`, `tfar_spatial_audio` or `tfar_geometry`); `append_own_voice` of
  `tfar_voice_pipeline_0_9` is documented as off-audio-thread only; both plugin apps cancel a
  push-to-talk release still waiting at game end (OF-031); OF-030 and F-091 are recorded.

## Owner decisions

- **2026-10-02: no live tests for now.** No Arma, Proton or TeamSpeak runs; OC-ts and OC-arma wait.
- **2026-10-02: the TeamSpeak SDK headers stay** in `crates/teamspeak/tfar_ts3_host/sdk_headers/`
  (the repository is private); `language-bans` admits `.h` files there only.
- **2026-10-02: the HRIR download is approved** for P3.
- **2026-10-02: no `ai guard` subcommand is needed** in xtask (OF-012).

## Open findings

Ids use the `OF-` prefix, because `F-` ids are taken by the
[bug fixes](/documentation/program/bug_fixes.md). Each finding is triaged FIX (to be done, with the
slice that does it), NOTE (recorded, no change needed now) or CLOSE (resolved).

- **OF-001 (NOTE): the plan's bug catalogue lists two 1.0-only bugs under "Both versions".** The
  LIFO async queue exists only in 1.0 (0.9.12 has no async path,
  `reference/tfar_1_0/ts/src/SharedMemoryHandler.cpp:14`), and the nickname hash index exists only
  in 1.0 (0.9.12 keys a `std::map` by nickname, `reference/tfar_0_9_12/ts/src/plugin.cpp:408`).
  bug_fixes.md groups them under "1.0 only" (F-007, F-008). The plan's "POS and FREQ without length
  checks" holds for 1.0; 0.9.12 checks the count but crashes on malformed numbers, so F-003 covers
  both forms. Reported by A2.
- **OF-002 (CLOSE): the installed plugin file name decides the TeamSpeak plugin name,
  and the original receivers filter plugin commands by that name.** 0.9 accepts only senders whose
  name starts with `task_force_radio` (`reference/tfar_0_9_12/ts/src/plugin.cpp:3116`), 1.0 only
  `TFAR`. The dist names planned in the A4 brief (`libtfar_plugin_<v>_linux_amd64.so`) would give
  plugin names that neither original accepts, so Windows players would ignore our tangent and volume
  commands and the reverse. Verify in M2 with the mock and two clients; the likely fix is dist names
  `task_force_radio_linux_amd64.so` (0.9) and `TFAR_linux_amd64.so` (1.0), which also keeps the two
  Rust plugins apart. Reported by A2. Closed on 2026-10-02 with the decision "dist names
  `task_force_radio_linux_amd64.so` / `TFAR_linux_amd64.so`", recorded in the
  [workspace decisions](/documentation/decisions/workspace.md); `cargo xtask dist` and the install
  runbook use those names.
- **OF-003 (NOTE): "foundation depends on nothing" (plan §4) conflicts with the edge
  `tfar_arma_values` → `tfar_geometry` in the shared brief.** laws_and_gates.md reads the rule as
  "foundation depends only on foundation"; A4's `crate-tiers` must implement that reading. Reported
  by A2.
- **OF-004 (NOTE): `dependency-firewalls` is its own gate in the A4 brief, while plan §4 lists the
  firewalls under `crate-tiers`.** laws_and_gates.md lists it as a separate gate that `ci-local`
  runs. Reported by A2.
- **OF-005 (NOTE): plan §4 allows C/C++ only in the golden-vector shim folder; D9 and the A4 brief
  also need the oracle folder** (vendored F3DAudio.c). laws_and_gates.md allows both. Reported by A2.
- **OF-006 (CLOSE): the plan does not say how the plugin picks the `-tfdebug` port.** The
  [game link protocol](/documentation/architecture/game_link_protocol.md#ports) proposes an
  environment variable in TeamSpeak's environment, matching the original's compile-time switch; the
  roadmap lists it as an open question for the owner. Reported by A2. Closed on 2026-10-03: both
  plugins listen on the debug port when `TFAR_GAME_LINK_DEBUG=1` (L3, 52bda9e), recorded in the
  game link protocol as the default, pending the owner. Superseded on 2026-10-03 by D24: each plugin
  listens on both its ports, and `TFAR_GAME_LINK_DEBUG` goes (V6 and W7 remove it from the apps).
- **OF-007 (NOTE): the 0.9 behaviour spec cites line 218 of the clunk hrtf.cpp for the disabled KEMAR
  filter; the commented line is 216 in both trees.** The spec is a frozen record and stays as is;
  the program records cite 216. Reported by A2.
- **OF-008 (NOTE, owner triage): bugs in the behaviour specs that the plan's catalogue does not
  list.** Candidates for new F-ids: 1.0 `NetID` runs `strchr` on a non-terminated `string_view`
  (`reference/tfar_1_0/ts/src/datatypes.hpp:190`); the 1.0 debug message log grows without bound
  (`reference/tfar_1_0/ts/src/clientData.cpp:380`); 1.0 SPEAKERS with fewer than 6 parts aborts every
  later speaker (now F-029, fixed in 654db3b). Moot with the new transport: the 0.9 `Pipe error <n>` length bug, the 0.9 pipe
  handle leaked on reopen, and the 1.0 2044-byte request limit. Reported by A2.
- **OF-009 (NOTE): plan §8 puts a root Cargo.toml with empty members in S0; the S0 commit has none.**
  A3 owns the root manifest, so nothing is lost. Reported by A2.
- **OF-010 (NOTE): plan §7 lists four game-facing strings; the 0.9 original has a fifth,
  `Pipe was closed by TS`.** The game link protocol keeps it for a 0.9 call whose connection the
  plugin closes mid-call. Reported by A2.
- **OF-011 (CLOSE): switching mods within one TeamSpeak session leaves the first plugin active.**
  The dormant-plugin rule (plan §7) wakes a plugin on its first game message and never puts it back
  to sleep, so after playing 0.9 then 1.0 without restarting TeamSpeak, the 0.9 plugin keeps writing
  its out-of-game metadata beside the 1.0 one. Options for the owner: accept it (restart TeamSpeak
  between mods), or let a plugin return to dormant once its game-end handling has finished.
  Reported by A2. Closed on 2026-10-03 by D25: a plugin returns to dormant after its game-end
  handling; V6 and W7 implement it.
- **OF-012 (NOTE): a host hook runs `target/debug/xtask ai guard` before every tool call.** Our xtask
  has no `ai guard` command, so once a build leaves the binary at that path every Bash and Read call
  is blocked. A4 once replaced the built xtask binary with a stub to get past it. The rule since:
  every agent builds into `CARGO_TARGET_DIR=<repo>/target-<agent id>` (the
  [shared brief](/documentation/program/agent_briefs/shared_brief.md#build-directory-rule)), and
  never stubs, renames or deletes files to pass a hook. The owner said no `ai guard` subcommand is
  needed. Reported by A4 and the orchestrator.
- **OF-013 (NOTE): `tfar_ts3_mock` depends on `libc`.** The mock plays TeamSpeak and hands the
  plugin `malloc`ed memory that the plugin frees through `freeMemory`, so C0 widened the `libc`
  firewall from `tfar_ts3_host` alone to `tfar_ts3_host` and `tfar_ts3_mock` (decision entry "The
  library map gains libc for TeamSpeak memory and libloading for the probe (D18)"). Reported by C0.
- **OF-014 (CLOSE): `cargo deny check` fails on two new dependencies.** `webpki-roots`
  1.0.9 (through `ureq`, `tfar_web_requests`) is `CDLA-Permissive-2.0`, missing from the licence
  allow-list; `paste` 1.0.15 (through `sci-rs`, `tfar_dsp_filters`) is unmaintained,
  RUSTSEC-2024-0436. Both need a deny.toml entry with a reason. Reported by C0. Closed by 8c87196.
- **OF-015 (CLOSE): `typos` fails on the TeamSpeak SDK spellings.** The `ts3plugin-sys`
  field for `requestUnmuteClients`, whose name carries a transposed "request"
  (`crates/teamspeak/tfar_ts3_host/src/ts3_functions/client_actions.rs:83`,
  `crates/teamspeak/tfar_ts3_host/src/tests/sdk_layout.rs:91`,
  `crates/teamspeak/tfar_ts3_host/src/ts3_functions/tests/ts3_calls.rs:308`), and the upstream
  comments of `crates/teamspeak/tfar_ts3_host/sdk_headers/public_definitions.h` (three misspelt
  words at lines 13, 164 and 414). typos.toml needs that field spelling as an accepted word and the
  sdk_headers folder excluded. Reported by C0. Closed by 8c87196.
- **OF-016 (NOTE): the M5 and M7 briefs use other agent ids than the plan's rosters.** The plan names
  M5 W1 and W2 and M7 U1 to U8; the briefs are S1 (`m5_s1_s2_…`) and W1 (`m7_w1_…`). The briefs are
  the ids in use. Reported by C0.
- **OF-017 (CLOSE): V1 found the brief and the C++ disagree on trailing empty fields.** The V1 brief
  gave VERSION exactly 4 tokens and TANGENT exactly 5, as the spec table does; the original splits
  with `getline`, which drops a trailing empty field, so an empty serious-channel password or an
  empty radio subtype makes the original refuse the message. Fixed as F-027 and F-028 in 61f66f6.
  Reported by V1.
- **OF-018 (CLOSE): W1 and S1 found original bugs the catalogue did not list.** 1.0 SPEAKERS stops at
  the first short record (F-029, 654db3b); the 1.0 Piwik custom-variable parser loops forever on
  text that ends inside a variable (F-030, 420d37e). Both are now in the
  [bug fixes](/documentation/program/bug_fixes.md). Reported by W1 and S1.
- **OF-019 (CLOSE): W1 refuses a text value on a boolean or number setting, which the brief did not
  name as a fix.** The original's `setString` writes through the wrong member of the setting's union
  (`reference/tfar_1_0/ts/src/settings.hpp:96`), so the change is a bug fix; R1 recorded it as F-031.
  W1 also leaves `RECV_FREQS` unmapped, as the original's hash switch does, although the spec lists
  it. Reported by W1.
- **OF-020 (FIX, W3 or the next slice touching `tfar_arma_values`): `split_keeping_every_field` is
  crate-private in `tfar_arma_values`.** The 1.0 split (`helpers::split`, every field kept) is needed
  by `tfar_game_commands_1_0`, which carries its own crate-private copy over `str::split`
  (`crates/version_1_0/tfar_game_commands_1_0/src/fields/message_fields.rs:30`) and calls
  `str::split` directly for speaker records. Make the `tfar_arma_values` function public and use it
  in the 1.0 crates, so the rule lives in one place. Reported by W1.
- **OF-021 (NOTE, owner): the IRCAM LISTEN terms.** IRCAM's pages state no licence for the LISTEN
  data; the sphere ships with the IRCAM credit in `assets/hrir/CREDITS.md` and is never edited. The
  owner decides before release (M8, with D16) whether that is enough to redistribute it in the dist
  folders. Reported by P3. It stays a NOTE: the owner checks the IRCAM terms before any
  redistribution of the dist folders.
- **OF-022 (CLOSE): a makeup gain after the HRTF.** The compensated sphere is diffuse-field
  equalised, so direct speech comes out about 4 dB quieter in front and 9 dB behind. Open question
  in the [audio pipeline](/documentation/architecture/audio_pipeline.md#open-questions); settle in
  M6 or M7 before the voice pipelines land. Reported by P3. Closed on 2026-10-03 by D23: +4 dB on
  direct speech, none on speaker radios or the ILD fallback.
- **OF-023 (CLOSE): HRTF memory.** Each `HrtfSource` copies the prepared processor, about
  2 MB, so memory grows by about 2 MB per audible source; the voice pipelines measure it and decide
  whether a cap or a shared processor is needed. Reported by P3. Closed on 2026-10-03 by D26: the
  memory is accepted, with no pooling.
- **OF-024 (NOTE): the mock's fake-state lock is re-entrant over a `RefCell`.** A fake that is still
  using the state when the plugin calls another fake fails at once on the `RefCell` borrow instead
  of deadlocking (`tools/tfar_ts3_mock/src/fake_state/host_session.rs`), so scenario code must never
  hold the state across a call into the plugin. Reported by H2.
- **OF-025 (NOTE): TFAR's 1.0 voice curve runs to distance 1.25, outside X3DAudio's documented 0 to 1
  point range.** F3DAudio computes the points past 1 anyway and the port matches it (G1); how
  Windows X3DAudio behaves there cannot be verified on this machine. Reported by G1.
- **OF-026 (CLOSE): CLAUDE.md's directory atlas still says `tfar_golden_vectors` makes
  "fixtures".** The crate runs live oracle comparisons in its tests; R1 may edit only law 12 of
  CLAUDE.md, so the atlas line (`tools/tfar_golden_vectors/`) waits for the orchestrator. Reported
  by R1. Closed: the atlas line now reads "oracles compared live against the audio crates".
- **OF-027 (CLOSE): the 0.9 loudspeakers get the HRTF makeup gain, against D23.** `place_sound`
  (`crates/version_0_9/tfar_voice_pipeline_0_9/src/voice_chain/sound_placement.rs:29`) applies
  `hrtf_makeup_gain()` to every HRTF placement, and the loudspeaker path calls it too
  (`crates/version_0_9/tfar_voice_pipeline_0_9/src/voice_chain/radio_reception.rs:145`); D23 gives
  speaker radios none, as the 1.0 pipeline does. Reported by R3. Closed by P5 (b7f2efd): the
  0.9 loudspeaker path renders through the HRTF at unity (`a_loudspeaker_through_the_hrtf_gets_no_makeup_gain`).
- **OF-028 (CLOSE): the plugin app READMEs still document `TFAR_GAME_LINK_DEBUG`.**
  `crates/version_0_9/tfar_plugin_0_9/README.md:63`, `crates/version_1_0/tfar_plugin_1_0/README.md:54` and the `listen_port.rs`
  lines of both `src/README.md` describe the variable D24 removes; they change with the D24 code in
  wave 2. Reported by R3. Closed by V6 (110de0e) and W7 (0ced2f5): no README or source names the variable
  any more.
- **OF-029 (NOTE): F-002 has no 1.0 test.** The 1.0 state is plain data behind the app's lock, so
  the borrow checker holds the fix; the [bug fixes](/documentation/program/bug_fixes.md) name the
  0.9 `Send` test and leave the 1.0 half pending. Reported by R3.
- **OF-030 (NOTE): a web request in flight can outlive the plugin's shutdown.** `ureq` cannot abort a
  request, so shutdown cancels it (its answer is dropped) and leaves its thread to end within the
  request's own timeout, up to 10 s for the update check and 5 s for a Piwik request
  (`a_web_request_in_flight_never_holds_the_shutdown_past_two_seconds` for 0.9,
  `shutdown_joins_every_thread_the_plugin_started_within_two_seconds` for 1.0). The plugin
  acts on nothing in that time. Whether TeamSpeak may `dlclose` the plugin while such a thread runs
  is unproven without a live test. Reported by the orchestrator (C1-5).
- **OF-031 (CLOSE): a delayed push-to-talk release still waiting at game end broadcast after the
  plugin was dormant again, against D25.** Both apps now cancel it at game end: 0.9 restores voice
  activation and the microphone at once without the broadcast
  (`a_game_end_cancels_a_waiting_release_so_a_dormant_plugin_never_broadcasts_it`); 1.0 moves the
  tangent gate's release generation on, so a waiting job does nothing, and its game end restores
  the microphone as before (`a_game_end_cancels_a_waiting_release_so_a_dormant_plugin_never_broadcasts_it`,
  `a_game_end_cancels_every_waiting_release_and_the_next_press_starts_afresh`, and the mock step in
  `missionend_disconnects_with_the_off_sound_and_needcfg_follows_setcfg`). Reported by the
  orchestrator (C1-4); fixed by C1.
- **OF-032 (FIX, the next slice on `tfar_voice_pipeline_1_0`): `append_own_voice_for_speakers` uses
  the blocking mixer append.** The 1.0 app pushes its own voice through a `StreamProducer` and no
  longer calls it (`crates/version_1_0/tfar_voice_pipeline_1_0/src/voice_processor.rs:102`); like
  `append_own_voice` of 0.9 (documented by C1 as off-audio-thread only), it needs the same
  documentation or a `StreamProducer` parameter. Reported by C1.
- **OF-033 (CLOSE): after TeamSpeak's "Reload" the plugin found no plugin folder.** Found in the
  owner's first live run (the 0.9 plugin in the Flatpak TeamSpeak 3.6.2): the first load was fine;
  after Reload the log said "TeamSpeak named no plugin folder; the radio sounds are off" with
  `getPluginPath` returning an empty path, and the HRIR sphere failed as a relative path. TeamSpeak
  keeps the `.so` loaded on Reload but registers a new plugin ID; `tfar_ts3_host` kept the first ID
  in a `OnceLock`, so `getPluginPath` (and `sendPluginCommand`) went out under the stale one. Fixed
  by H5 as [F-101 and F-102](/documentation/program/bug_fixes.md#found-in-live-runs): the latest ID
  wins, and `Ts3::plugin_folder` falls back to the loaded library's folder when `getPluginPath`
  names none. The mock host now has `MockHost::reload` and answers a stale ID with an empty path;
  each app's `after_a_reload_the_plugin_finds_its_folder_and_sends_under_the_new_id` fails on the
  old code with the owner's empty-path error and holds on the new.

## Handoff

All code is written and every milestone is code-complete: every library crate, both extensions,
both full plugin apps (in `crates/version_0_9/` and `crates/version_1_0/` since 2026-10-10;
0.9 at 110de0e and 1.0 at 0ced2f5 on the shared library APIs, D24 and D25 in place), the mock
host, the probe and the golden-vector oracle. C1 closed M8 in code on 2026-10-03:
`cargo xtask ci ci-local` held all 21 steps, `cargo xtask dist` produced both version folders (the
artefact table is in the execution log), the [roadmap](/documentation/roadmap.md) is ticked, the
[install runbook](/documentation/runbooks/manual_install_and_test.md) follows the real dist layout,
and every [bug fix](/documentation/program/bug_fixes.md) names its test except the 1.0 half of
F-002 (OF-029).

1. **Orchestrator:** commit C1 and put its hash in Current state (Last green commit) and on the M0
   Close and C1 rows.
2. **Owner, live checks only:** OC-ts (both plugins load in the Flatpak TeamSpeak), OC-arma (the
   probe's `PONG` under Proton's wine, then `TS_INFO\tPING` in Arma with BattlEye off) and
   OC-install (play both mods), each from the install runbook; the `hrtf` click question of the
   roadmap is part of the listening.
3. **Owner, before any release:** the licence (D16) and the IRCAM terms of the sphere (OF-021).
4. Open FIX findings, small and outside the live checks: OF-020 (`split_keeping_every_field` made
   public in `tfar_arma_values`) and OF-032 (`append_own_voice_for_speakers` of 1.0). OF-029 and
   OF-030 stay NOTEs.
