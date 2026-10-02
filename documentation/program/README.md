**Status:** live

# Rust port program records

The live records of the program that rewrites both TFAR versions in Rust: the plan, the crates, the
laws and gates, the bug fixes, the progress tracker, and the frozen research the plan rests on. Any
session, including a fresh one with no chat history, resumes the program from this folder.

## Contents

```text
documentation/program/
├── agent_briefs/        the shared agent brief and one prompt per agent slice
├── behaviour_specs/     frozen specs of the original plugins and extensions, bugs included
├── bug_fixes.md         every bug the port fixes: original code, old and new behaviour, owner, test
├── crate_catalogue.md   every crate: tier, the original code it ports, externals, fixes, milestone
├── laws_and_gates.md    edge matrix, firewalls, unsafe allow-list, features, gates, perturbations
├── program_plan.md      context, verified findings, decisions, execution model, rosters, risks
├── progress.md          current state, agent rows, execution log, amendments, findings, handoff
└── research/            frozen surveys of the conventions and libraries the plan adopted
```

## How it works

To resume, read in this order:

1. [progress.md](/documentation/program/progress.md): Current state and Handoff name the milestone,
   the last green commit and the exact next step.
2. [program_plan.md](/documentation/program/program_plan.md): the binding decisions D1–D22, the
   execution model and the agent roster with budgets for each milestone.
3. [crate_catalogue.md](/documentation/program/crate_catalogue.md): what each crate ports and when.
4. [laws_and_gates.md](/documentation/program/laws_and_gates.md): what every slice and milestone
   must pass.

One orchestrator session runs each milestone: it writes the agent prompts into
[agent_briefs](/documentation/program/agent_briefs/README.md), launches Opus agents on disjoint
files, reviews and gates each slice, commits it, and updates progress.md after every report. The
milestones themselves, with their checklists and acceptance tests, are in the
[roadmap](/documentation/roadmap.md).

## Code

- The four apps, the library crates and the tools (planned: apps/, crates/, tools/) are what these
  records plan and track; `reference/` holds the original code they port.

## Boundaries

- Depends on: the approved plan of 2026-10-02, the behaviour specs and research in this folder, and
  the reference trees under `reference/`.
- Used by: the roadmap, the runbooks, every agent brief, and the xtask layout module that mirrors
  laws_and_gates.md.
- Rules: behaviour_specs and research are frozen records and are never edited; progress.md is the
  only tracker; bug-fix ids and open-finding ids are never reused.

## Related documentation

- [Roadmap](/documentation/roadmap.md) — the milestones M0–M8 with checklists and acceptance.
- [Architecture](/documentation/architecture/README.md) — the workspace layout, game link and audio
  pipeline the crates implement.
- [Runbooks](/documentation/runbooks/README.md) — manual install and resuming in Claude Code.
