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
| Current milestone | M0 — Workspace skeleton, reference archive, gates |
| Last green commit | — (no gate runs before A4 builds the xtask tool; S0 baseline is de41c9b) |
| Next action | Review the A1 and A2 reports and diffs, commit both slices, then launch A3 from its brief |
| Blocked on | nothing |

Row format: `- [ ] <ID> (<budget>) <role> — status — commit — notes`. A status is `pending`,
`running`, `done`, `blocked` or `fix-list`. Owner checkpoints are rows of their own. Budgets: S =
150k, M = 250k, L = 350k tokens.

## Milestones

### M0 Workspace skeleton

- [x] S0 (orchestrator) git init, tree moves, toolchain and tools, specs and briefs — done — de41c9b — see the execution log
- [ ] A1 (M) standards, templates, CLAUDE.md and AGENTS.md, decisions, glossary, reference README — running — — in parallel with A2
- [ ] A2 (M) program records, roadmap, architecture, runbooks — done, awaiting review and the slice commit — — open findings OF-001 to OF-011
- [ ] A3 (M) root config and all 38 crate skeletons — pending — — starts after A1's templates
- [ ] A4 (L) xtask gates, `ci ci-local`, `dist`, `new-crate`, `reference-manifest` — pending — — starts after A3; budget per amendment A1
- [ ] Close (orchestrator) full `ci-local`, Windows cross-build, commits per slice, handoff, memory note — pending — —

### M1–M8

Rows are added when each milestone starts, from the proposed rosters in the
[program plan](/documentation/program/program_plan.md#5-milestones-and-agents).

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

## Amendments

- **A1 (2026-10-02): A4's budget is L (350k), not M.** The plan's M0 roster gave A4 an M budget; the
  gate set (16 repository gates, each with a perturbation test, plus `ci`, `dist`, `new-crate` and
  `reference-manifest`) needs L. The [A4 brief](/documentation/program/agent_briefs/m0_a4_xtask_gates.md)
  already says L.

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
- **OF-002 (FIX, M2 H4 and M8 R1): the installed plugin file name decides the TeamSpeak plugin name,
  and the original receivers filter plugin commands by that name.** 0.9 accepts only senders whose
  name starts with `task_force_radio` (`reference/tfar_0_9_12/ts/src/plugin.cpp:3116`), 1.0 only
  `TFAR`. The dist names planned in the A4 brief (`libtfar_plugin_<v>_linux_amd64.so`) would give
  plugin names that neither original accepts, so Windows players would ignore our tangent and volume
  commands and the reverse. Verify in M2 with the mock and two clients; the likely fix is dist names
  `task_force_radio_linux_amd64.so` (0.9) and `TFAR_linux_amd64.so` (1.0), which also keeps the two
  Rust plugins apart. The install runbook names the files generically until this settles. Reported by
  A2.
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

## Handoff

M0 is in progress. S0 is committed (de41c9b). A1 (standards) and A2 (program records) ran in
parallel; A2's records are written. The next session:
1. Reviews the A1 and A2 diffs against their briefs, triages their findings here, and commits each
   slice (`docs(program): …` for A2) with explicit paths.
2. Decides OF-002 before A4 writes `dist`, or tells A4 to keep the brief's names until M2.
3. Launches A3 from its brief, then A4, then runs the M0 close from the
   [roadmap](/documentation/roadmap.md#m0--workspace-skeleton-reference-archive-gates).
