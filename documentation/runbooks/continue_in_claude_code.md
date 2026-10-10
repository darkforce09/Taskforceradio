**Status:** live

# Continue the program in Claude Code

How to start the next working session on the TFAR Rust port in Claude Code, with the prompt to
paste. The prompt is milestone-neutral: it sends the session to progress.md for the next step, so it
only changes when the way the program runs changes.

## Prerequisites

- Claude Code (the Code tab of Claude Desktop, or the `claude` command line), signed in.
- This repository at `/run/media/system/Main_storage/Projects/Taskforceradio`, with both
  reference trees present: `reference/tfar_1_0` (its own upstream clone, gitignored here) and
  `reference/tfar_0_9_12` (committed).
- The toolchain from `rust-toolchain.toml`, `cargo-deny` and `typos` installed (`cargo deny
  --version`, `typos --version`).

## Steps

1. Open Claude Code in the repository folder.

   ```bash
   cd /run/media/system/Main_storage/Projects/Taskforceradio
   claude
   ```

   **Expected:** the session starts in the repository and reads CLAUDE.md.

2. Paste the prompt below as the first message.

   **Expected:** Claude reads progress.md and the program plan, states the current milestone, the
   last green commit and the next action from the Handoff section, and shows its plan for that step
   before launching any agent.

## Prompt

```text
You're continuing the TFAR Rust port: a from-scratch Rust rewrite of both Task Force Arrowhead Radio
versions (0.9.12 and 1.0) so Arma 3 under Proton talks to the native Linux Flatpak TeamSpeak 3.
The repo is /run/media/system/Main_storage/Projects/Taskforceradio. Each mod gets a Rust TeamSpeak
plugin (.so, Linux x86_64) and a Rust Arma extension (task_force_radio_pipe_x64.dll, x86_64-pc-windows-gnu);
they talk over localhost TCP. The original mods sit read-only under reference/.

1. Read first: CLAUDE.md (the laws are binding), then documentation/program/progress.md (Current
   state and Handoff first), documentation/program/program_plan.md (decisions D1-D22, execution
   model, the agent roster for the current milestone), documentation/roadmap.md,
   documentation/program/crate_catalogue.md, documentation/program/laws_and_gates.md and
   documentation/program/bug_fixes.md. For the milestone at hand also read the matching
   behaviour spec in documentation/program/behaviour_specs/ and the architecture documents.

2. You are the orchestrator. For the current milestone: write each agent's prompt into
   documentation/program/agent_briefs/ as m<N>_<agent>_<role>.md (Goal, Steps, Owned files, Checks,
   Report additions, budget line), keep shared_brief.md current, launch Opus sub-agents on disjoint
   files (parallel only where the roster marks it), review every report and diff, run the fast stage
   gate per slice, commit each slice yourself, and update progress.md after every report.

3. Gates: the stage gate per slice is cargo fmt --all --check, clippy and tests of the touched
   crates with -D warnings, and the documentation gates (cargo xtask verify readme-coverage,
   readme-sections, status-lines, markdown-placement, link-check). The milestone close runs
   cargo xtask ci ci-local in full, ticks the roadmap and writes the Handoff.

4. Laws you must not bend: reference/ is never written (cargo xtask verify reference-untouched);
   Rust only, C/C++ only as golden-vector oracles; version_0_9 and version_1_0 crates never depend
   on each other; no panics in production code; unsafe only in the allow-listed FFI crates; no
   script copies anything into TeamSpeak or a mod folder; Conventional Commits straight to main
   with explicit paths and the Co-Authored-By trailer; no silent deferrals.

5. Show me the plan for the next action in progress.md first. Ask before downloading anything new
   (the IRCAM HRIR sphere among them), adding a dependency outside the library map in the program
   plan, or deciding an open question in the roadmap.
```

## Verify

After the session: `git log --oneline` shows one commit per finished slice, progress.md's Current
state and Handoff name the next step, and the roadmap boxes of finished work are ticked in the same
commits as the work.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `reference/tfar_0_9_12` is missing after a clone | It is gitignored: its LFS pointer stubs reference objects that exist nowhere, which GitHub refuses (D15) | Copy the 0.9.12 tree in from the owner's local copy, add the override shown below to `.git/info/attributes`, and check that `cargo xtask verify reference-untouched` holds |
| `reference/tfar_1_0` is missing after a clone | It is an upstream clone with its own `.git` and is gitignored in this repository (D15) | `git clone https://github.com/michail-nikolaev/task-force-arma-3-radio reference/tfar_1_0`, then `git -C reference/tfar_1_0 checkout fce977fa`; `cargo xtask verify reference-untouched` must hold |
| `cargo xtask verify reference-untouched` fails | A file under `reference/` changed, or the 1.0 clone is at another commit | Restore the file from the owner's local copy (0.9.12) or check out `fce977fa` again (1.0); never regenerate the manifest to make the gate pass |
| The Windows clippy or build step fails to link | `x86_64-w64-mingw32-gcc` is not installed | Install the mingw-w64 GCC cross compiler from the distribution's packages |

The LFS override for `.git/info/attributes`, one line:

```text
reference/** -filter -diff -merge -text
```

## Related documentation

- [Progress](/documentation/program/progress.md) — the current state and handoff the prompt starts
  from.
- [Program records](/documentation/program/README.md) — the resume reading order.
- [Manual install and test](/documentation/runbooks/manual_install_and_test.md) — the owner's
  checkpoints the program pauses for.
