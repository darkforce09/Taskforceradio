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
| Current milestone | M0 — Workspace skeleton, reference archive, gates (closing); M1 done; M2, M3 and M4 slices running in parallel |
| Last green commit | — (`cargo xtask verify all` failed at 2ff1886 on the gate debts C0 clears; C0's commit is the first candidate) |
| Next action | Review the C0 report and commit it; then review and commit H2, L3, P1, P3 and P4 as each reports, and rerun `cargo xtask ci ci-local` |
| Blocked on | nothing |

Row format: `- [ ] <ID> (<budget>) <role> — status — commit — notes`. A status is `pending`,
`running`, `done`, `blocked` or `fix-list`. Owner checkpoints are rows of their own. Budgets: S =
150k, M = 250k, L = 350k tokens.

## Milestones

### M0 Workspace skeleton

- [x] S0 (orchestrator) git init, tree moves, toolchain and tools, specs and briefs — done — de41c9b — see the execution log
- [x] A1 (M) standards, templates, CLAUDE.md and AGENTS.md, decisions, glossary, reference README — done — 5b2b918 — ran in parallel with A2
- [x] A2 (M) program records, roadmap, architecture, runbooks — done — 9a71a5a — open findings OF-001 to OF-011
- [x] A3 (M) root config and all 38 crate skeletons — done — 530eb13 — started in parallel with A1 (amendment A2)
- [x] A4 (L) xtask gates, `ci ci-local`, `dist`, `new-crate`, `reference-manifest` — done — 3046373 — budget per amendment A1; see OF-012
- [ ] C0 (M) gate cleanup and the M0 close records — running — — added by amendment A6; [brief](/documentation/program/agent_briefs/m0_c0_gate_cleanup.md)
- [ ] Close (orchestrator) full `ci-local`, Windows cross-build, commits per slice, handoff, memory note — pending — — after C0 and the running slices

### M1 Foundations

- [x] B1 (M) `tfar_geometry` and `tfar_arma_values` — done — 45a584d — ran early (amendment A3)
- [x] B2 (S) `tfar_sample_buffer` and the tracing rule — done — 86b4891 — ran early (amendment A3)

### M2 TeamSpeak host

- [x] H1 (L) `tfar_ts3_host` on `ts3plugin-sys`, layout tests against the API 26 headers — done — 2ff1886 — SDK headers kept in `crates/teamspeak/tfar_ts3_host/sdk_headers/`
- [ ] H2 (L) `tfar_ts3_mock` and `tfar_ts3_settings_db` (H3 folded in) — running — — [brief](/documentation/program/agent_briefs/m2_h2_mock_settings_db.md)
- [ ] H4 (S) metadata exports in both plugin stubs — running inside L3 — — the L3 brief covers the stubs' metadata
- [ ] OC-ts (owner) both stubs load in the Flatpak TeamSpeak — pending — — no live tests for now (owner decision of 2026-10-02)

### M3 Game link

- [x] L1 (M) `tfar_game_link_protocol` and `tfar_game_link_server` — done — 37dc8e0 — ran before M2 completed (amendment A4)
- [x] L2 (M) `tfar_arma_extension_abi` and `tfar_game_link_client` — done — 37dc8e0 — committed with L1
- [ ] L3 (L) both extension apps, the stub plugins with the game link, `tfar_extension_probe` — running — — [brief](/documentation/program/agent_briefs/m3_l3_apps_probe.md)
- [ ] OC-arma (owner) probe `PONG` under Proton's wine, then `TS_INFO\tPING` in Arma — pending — — no live tests for now

### M4 Audio primitives

- [ ] P1 (M) `tfar_dsp_filters` and `tfar_dynamics` (P2 folded in) — running — — [brief](/documentation/program/agent_briefs/m4_p1_p2_filters_dynamics.md)
- [ ] P3 (L) `tfar_hrtf` with the IRCAM sphere — running — — HRIR download approved by the owner; [brief](/documentation/program/agent_briefs/m4_p3_hrtf.md)
- [ ] P4 (M) `tfar_spatial_audio` and `tfar_playback_mixer` (P5 folded in) — running — — [brief](/documentation/program/agent_briefs/m4_p4_p5_spatial_playback.md)
- [ ] G1 (L) `tfar_golden_vectors` and the golden tests against the crates — pending — — after P1–P5 (amendment A5)

### M5–M8

Rows are added when each milestone starts, from the proposed rosters in the
[program plan](/documentation/program/program_plan.md#5-milestones-and-agents). Briefs for M5 S1, M6
V1 and M7 W1 are written (d8dee88).

## Execution log

| Date | Event | Detail | Logs |
|---|---|---|---|
| 2026-10-02 | S0 tree move | `task-force-arma-3-radio` → `reference/tfar_1_0` and `task-force-arma-3-radio-0.9.12` → `reference/tfar_0_9_12` as plain renames; verified byte-identical: 789 files (1.0, excluding `.git`) and 919 files (0.9.12), sha256 lists before and after matched | session scratchpad: pre_1_0.sha, pre_0_9.sha, premove_counts.txt |
| 2026-10-02 | S0 toolchain | Rust 1.99.0 with targets `x86_64-unknown-linux-gnu` and `x86_64-pc-windows-gnu`, cargo-deny 0.20.2 and typos 1.50.3 installed | session scratchpad: toolchain.log, tools_install.log |
| 2026-10-02 | S0 records | Behaviour specs (0.9 plugin, 1.0 plugin, extensions, Windows dependencies), research (conventions, libraries) and the M0 agent briefs saved under this folder | — |
| 2026-10-02 | S0 Git LFS | `reference/tfar_0_9_12` holds Git LFS pointer stubs (models, images) committed as plain bytes. LFS is neutralised locally through `.git/info/attributes` with the line `reference/** -filter -diff -merge -text`, so no LFS filter ever runs on the reference trees. A fresh clone must add the same line (see the [resume runbook](/documentation/runbooks/continue_in_claude_code.md#troubleshooting)) | — |
| 2026-10-02 | S0 commit | `chore(workspace): s0 baseline with reference trees and program specs` — de41c9b | — |
| 2026-10-02 | M0 launch | A1 and A2 launched in parallel from their briefs; A4's budget raised to L (amendment A1) | — |
| 2026-10-02 | A2 report | Roadmap, program records (plan, catalogue, laws and gates, bug fixes, progress, folder READMEs), architecture (layout, game link protocol, audio pipeline) and runbooks written; 26 bug-fix entries (6 both, 12 1.0-only, 8 0.9-only) | target/agent-logs (A2) |
| 2026-10-02 | A1, A2 commits | `docs(workspace): standards, templates, decisions and CLAUDE.md` — 5b2b918; `docs(program): roadmap, program plan, catalogue, laws and runbooks` — 9a71a5a | — |
| 2026-10-02 | A3 commit | `chore(workspace): root config and 37 crate skeletons` — 530eb13 (xtask is the 38th crate, A4's) | target/agent-logs (A3) |
| 2026-10-02 | B1, B2 commits | `feat(tfar-sample-buffer): …` — 86b4891; `feat(tfar-geometry): positions, directions and Arma value parsing` — 45a584d | target-m1-b1, target-m1-b2 |
| 2026-10-02 | A4 commit | `build(xtask): verify gates, ci-local lane, dist, new-crate, manifest` — 3046373 | target-a4 |
| 2026-10-02 | L1, L2 commit | `feat(tfar-game-link): protocol, plugin server and extension client` — 37dc8e0 | target-m3-l1, target-m3-l2 |
| 2026-10-02 | Briefs | M1–M4 briefs — f451d8d; C0, H2, L3, P3 briefs and the build directory rule — c38c7c0; S1, V1, W1 briefs — d8dee88 | — |
| 2026-10-02 | H1 commit | `feat(tfar-ts3-host): safe TeamSpeak layer over ts3plugin-sys` — 2ff1886 | target-m2-h1 |
| 2026-10-02 | Launch | C0, H2, L3, P1, P3 and P4 launched in parallel, each building into its own `target-<agent id>` folder | — |
| 2026-10-03 | C0 run | Gate debts cleared (language-bans, firewalls, link-check, readme-coverage, unsafe allow-list); `ci ci-local` stops at fmt on an L3 file, so each step was rerun alone | target/agent-logs/c0-*.log |

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
- **OF-006 (FIX, M3 L1): the plan does not say how the plugin picks the `-tfdebug` port.** The
  [game link protocol](/documentation/architecture/game_link_protocol.md#ports) proposes an
  environment variable in TeamSpeak's environment, matching the original's compile-time switch; the
  roadmap lists it as an open question for the owner. Reported by A2.
- **OF-007 (NOTE): the 0.9 behaviour spec cites line 218 of the clunk hrtf.cpp for the disabled KEMAR
  filter; the commented line is 216 in both trees.** The spec is a frozen record and stays as is;
  the program records cite 216. Reported by A2.
- **OF-008 (NOTE, owner triage): bugs in the behaviour specs that the plan's catalogue does not
  list.** Candidates for new F-ids: 1.0 `NetID` runs `strchr` on a non-terminated `string_view`
  (`reference/tfar_1_0/ts/src/datatypes.hpp:190`); the 1.0 debug message log grows without bound
  (`reference/tfar_1_0/ts/src/clientData.cpp:380`); 1.0 SPEAKERS with fewer than 6 parts aborts every
  later speaker. Moot with the new transport: the 0.9 `Pipe error <n>` length bug, the 0.9 pipe
  handle leaked on reopen, and the 1.0 2044-byte request limit. Reported by A2.
- **OF-009 (NOTE): plan §8 puts a root Cargo.toml with empty members in S0; the S0 commit has none.**
  A3 owns the root manifest, so nothing is lost. Reported by A2.
- **OF-010 (NOTE): plan §7 lists four game-facing strings; the 0.9 original has a fifth,
  `Pipe was closed by TS`.** The game link protocol keeps it for a 0.9 call whose connection the
  plugin closes mid-call. Reported by A2.
- **OF-011 (NOTE, M7): switching mods within one TeamSpeak session leaves the first plugin active.**
  The dormant-plugin rule (plan §7) wakes a plugin on its first game message and never puts it back
  to sleep, so after playing 0.9 then 1.0 without restarting TeamSpeak, the 0.9 plugin keeps writing
  its out-of-game metadata beside the 1.0 one. Options for the owner: accept it (restart TeamSpeak
  between mods), or let a plugin return to dormant once its game-end handling has finished.
  Reported by A2.
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
- **OF-014 (FIX, root config): `cargo deny check` fails on two new dependencies.** `webpki-roots`
  1.0.9 (through `ureq`, `tfar_web_requests`) is `CDLA-Permissive-2.0`, missing from the licence
  allow-list; `paste` 1.0.15 (through `sci-rs`, `tfar_dsp_filters`) is unmaintained,
  RUSTSEC-2024-0436. Both need a deny.toml entry with a reason. Reported by C0.
- **OF-015 (FIX, root config): `typos` fails on the TeamSpeak SDK spellings.** The `ts3plugin-sys`
  field for `requestUnmuteClients`, whose name carries a transposed "request"
  (`crates/teamspeak/tfar_ts3_host/src/ts3_functions/client_actions.rs:83`,
  `crates/teamspeak/tfar_ts3_host/src/tests/sdk_layout.rs:91`,
  `crates/teamspeak/tfar_ts3_host/src/ts3_functions/tests/ts3_calls.rs:308`), and the upstream
  comments of `crates/teamspeak/tfar_ts3_host/sdk_headers/public_definitions.h` (three misspelt
  words at lines 13, 164 and 414). typos.toml needs that field spelling as an accepted word and the
  sdk_headers folder excluded. Reported by C0.
- **OF-016 (NOTE): the M5 and M7 briefs use other agent ids than the plan's rosters.** The plan names
  M5 W1 and W2 and M7 U1 to U8; the briefs are S1 (`m5_s1_s2_…`) and W1 (`m7_w1_…`). The briefs are
  the ids in use. Reported by C0.

## Handoff

M0 is closing; M1 is done; M2, M3 and M4 run in parallel. Committed: S0, A1–A4, B1, B2, L1, L2, H1
and the briefs (last commit d8dee88). C0 has cleared the gate debts of 2ff1886; H2, L3, P1, P3 and P4
are running, and files of `tfar_game_commands_0_9` and `tfar_web_requests` are changing as well. The
next session:
1. Reviews the C0 diff and commits it with explicit paths.
2. Reviews and commits each running slice as it reports, and fixes OF-014 and OF-015 in deny.toml and
   typos.toml.
3. Runs `cargo xtask ci ci-local` on a quiet tree; when it holds, records the first green commit
   above and closes M0 from the [roadmap](/documentation/roadmap.md#m0--workspace-skeleton-reference-archive-gates)
   (the Windows cross-build and the dist placeholders included).
4. Launches G1 once P1–P5 are committed (amendment A5).
