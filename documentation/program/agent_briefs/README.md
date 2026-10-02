**Status:** live

# Agent briefs

The prompts the orchestrator gives its Opus sub-agents: one shared brief every agent reads first, and
one brief per agent slice.

## Contents

```text
documentation/program/agent_briefs/
├── m0_a1_standards.md             M0 A1: standards, templates, CLAUDE.md, decisions, glossary
├── m0_a2_program_records.md       M0 A2: program records, roadmap, architecture, runbooks
├── m0_a3_workspace_skeletons.md   M0 A3: root configuration and all 38 crate skeletons
├── m0_a4_xtask_gates.md           M0 A4: the xtask tool, every gate, ci, dist, scaffolding
└── shared_brief.md                rules, environment facts, crate edges and report format for every agent
```

## How it works

A slice brief is named `m<N>_<agent>_<role>.md`: the milestone number, the agent id in lowercase
(`a1`, `b2`, `u7`) and a snake_case role, so `m0_a2_program_records.md` is milestone M0, agent A2,
the program records. Each brief opens with "You are agent <ID>, …. Read the shared brief first",
then holds Goal, Steps, Owned files, Checks, Report additions and a closing budget line (S = 150k,
M = 250k, L = 350k tokens). The orchestrator writes a milestone's briefs at its start from the
proposed roster in the [program plan](/documentation/program/program_plan.md#5-milestones-and-agents),
and keeps the [shared brief](/documentation/program/agent_briefs/shared_brief.md) current before
each launch. Briefs of a finished milestone stay as the record of what each agent was asked.

## Code

- Each brief names the code paths its agent owns; this folder holds no code.

## Boundaries

- Depends on: the program plan, the crate catalogue, and laws and gates.
- Used by: the orchestrator and every agent it launches.
- Rules: an agent edits only its owned paths; amendments at launch are recorded in
  [progress](/documentation/program/progress.md), not by rewriting a brief already run.

## Related documentation

- [Progress](/documentation/program/progress.md) — which agents ran, their reports and amendments.
- [Resume runbook](/documentation/runbooks/continue_in_claude_code.md) — starting the next session.
