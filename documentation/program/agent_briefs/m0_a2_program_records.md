**Status:** live

# M0 · A2 · Program records, roadmap, architecture, runbooks

You are agent A2, the program-records author for the TFAR Rust port. Read `/documentation/program/agent_briefs/shared_brief.md` first.

## Goal
Turn the approved plan into the durable program records that every later milestone resumes from. Follow the
TBD-Reforger restructure record style and the TBD-subtitles roadmap style
(`/documentation/program/research/conventions_survey.md`). The approved plan is
`/home/Samuel/.claude/plans/i-want-to-edit-wobbly-squirrel.md`.

## Steps
1. **`/documentation/roadmap.md`.**
   - Write M0–M8 exactly as in plan §6.
   - Each milestone gets: an intro line, a `- [ ]` checklist (M0 items unticked; the orchestrator ticks them at close), and an `**Acceptance:**` paragraph.
   - Add the TBD-subtitles intro rules: tick items in the same commit as the work; a milestone is done when its acceptance passes; only the owner moves items.
   - End with a `## Open questions` table: | Question | Settle by |. Include: licence (D16, before release); whether `hrtf`'s click artefacts on fast-moving sources are acceptable (M4); whether 32-bit Arma matters (no, unless the owner says so).
2. **`/documentation/program/`.**
   - `README.md`: resume reading order is progress → plan → catalogue → laws.
   - **`program_plan.md`**, with these sections:
     - Context.
     - Verified findings. Number them; each carries evidence by path, e.g. `reference/tfar_1_0/ts/src/SharedMemoryHandler.cpp:271`. Take them from `/documentation/program/behaviour_specs/`.
     - Binding decisions D1–D22.
     - Execution model: per milestone, prompts are written into `agent_briefs/`, Opus agents run, the orchestrator reviews and gates, commits per slice, and the handoff is written. Fast stage gate per slice; full `ci-local` at the milestone close.
     - Milestone table: | Milestone | Agents and content | Extra gates, checkpoints |. M0 has the real roster (S0, A1 ∥ A2, A3, A4). M1–M8 are proposed rosters with budgets (S = 150k, M = 250k, L = 350k), to be refined at each milestone start.
     - Shared brief essentials.
     - Risks (plan §10).
   - **`crate_catalogue.md`.**
     - Start with an abbreviation table.
     - Then one section per category, with a table: | Crate | Tier | Ports from (reference file:line) | External crates | Fixes (F-ids) | Milestone |.
     - Use the edge table in the shared brief. Take the "ports from" references from the behaviour specs, e.g. `tfar_radio_effects_1_0` ← `reference/tfar_1_0/ts/src/RadioEffect.hpp`. Take external crates from plan §4a.
   - **`laws_and_gates.md`.** The laws and the gate set from plan §4 and §9, in full:
     - the edge matrix as a table;
     - the firewalls;
     - the unsafe allow-list;
     - the feature rules;
     - each gate's name, what it checks, and its pass/fail/could-not-run exit codes (0/1/2);
     - stage gate vs milestone gate;
     - the perturbation proofs required.
   - **`bug_fixes.md`.**
     - One entry per bug in plan §7, ids F-001…, grouped "Both versions", "1.0 only", "0.9 only".
     - Columns: | Id | Reference (file:line) | Original behaviour | Fixed behaviour | Owning crate | Milestone | Test |. The Test column says "pending" for now.
     - Take exact references and behaviours from the behaviour specs.
   - **`progress.md`**, with sections:
     - `## Current state`: Branch main, Current milestone M0, Last green commit —, Next action, Blocked on.
     - `## Milestones`: `- [ ] <ID> (<budget>) <role> — status — commit — notes` rows for S0, A1, A2, A3, A4 and the close.
     - `## Execution log`: | Date | Event | Detail | Logs |. Rows so far: tree move verified byte-identical (789 + 919 files, sha256 lists matched); toolchain 1.99.0 + cargo-deny 0.20.2 + typos 1.50.3 installed; specs saved.
     - `## Amendments`.
     - `## Open findings`.
     - `## Handoff`.
   - **READMEs** for `behaviour_specs/`, `research/` and `agent_briefs/`, each listing their files. The agent_briefs README explains the naming `m<N>_<agent>_<role>.md`.
3. **`/documentation/architecture/`.**
   - `README.md`.
   - **`workspace_layout.md`**: the tree, the categories with the "which folder does a new crate go in" rule, and an ASCII dependency diagram.
   - **`game_link_protocol.md`**, a full spec from plan §7 and `/documentation/program/behaviour_specs/extensions.md`:
     - ports, and the `-tfdebug` port (+1);
     - the frame layout and kinds, and the HELLO handshake;
     - per-flavour extension semantics. 0.9: everything is sync. 1.0: `~` async, DFRAME → NEEDCFG/OK, MISSIONEND, FOCUS;
     - the plugin's connected/disconnected rule and pluginTimeout;
     - timeouts and backoff;
     - every game-facing error string;
     - FIFO async ordering;
     - the dormant-plugin rule;
     - a sequence diagram in a `text` block.
   - **`audio_pipeline.md`**: the voice and radio processing chain per version, as it will be built. Use the libraries from plan §4a: `hrtf` for direction, the `tfar_spatial_audio` curve and cone, sci-rs + biquad filters, the compressor. Mark the fixed bugs with F-ids.
4. **`/documentation/runbooks/`.**
   - `README.md`.
   - **`manual_install_and_test.md`**, numbered steps, each with an `**Expected:**` line:
     - build with `cargo xtask dist`;
     - copy the `.so` and `radio-sounds/` into `~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/`, and remove the old `task_force_radio_win*.dll` / `TFAR_win*.dll` by hand;
     - copy `task_force_radio_pipe_x64.dll` to the mod root for the matching mod;
     - turn BattlEye off;
     - check in the TS plugin list and with `callExtension "TS_INFO\tPING"`;
     - run the probe under Proton's wine.
   - **`continue_in_claude_code.md`**: Prerequisites / Steps / Prompt (a pasteable `text` block) / Verify / Troubleshooting.

## Owned files
`/documentation/roadmap.md`, `/documentation/program/{README,program_plan,crate_catalogue,laws_and_gates,bug_fixes,progress}.md`,
`/documentation/program/{behaviour_specs,research,agent_briefs}/README.md`, `/documentation/architecture/**`,
`/documentation/runbooks/**`. Do not edit the existing spec, research or brief files; report fixes as findings.

## Checks
- Status lines; ≤ 500 lines per document (split into a subfolder if needed); links from the repository root.
- Every backticked path exists, or is written as plain text when it is planned.
- `typos` over your files.

## Report additions
F-id count per group; any contradiction found between the plan and the behaviour specs.

Budget: M (250k tokens). Stop there and report what is done and what is not.
