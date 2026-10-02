**Status:** live

# Decisions

The log of choices that shape the port and would otherwise be argued again: the workspace and its
conventions, the game link, the audio libraries, and what happens to the originals' bugs and features.
Each entry says what was decided, why, what follows, and which earlier entry it replaces. Questions not
yet decided live in the [roadmap](/documentation/roadmap.md)'s open questions.

## Contents

```text
documentation/decisions/
├── audio.md       HRTF through the hrtf crate, the distance and cone gain model, the filter libraries
├── bug_policy.md  fix every bug, keep every feature, C and C++ only as oracles, the port order
├── game_link.md   Proton only, the TeamSpeak bindings, the TCP game link, its ports, the focus frame
└── workspace.md   the rewrite, repository, names, layout, conventions, git, panics, dist, toolchain, libraries
```

## How it works

The log is one set of entries split over files by subject, each file within 500 lines. An entry is a
`###` heading, `<date> — <the decision>`, with **Context**, **Decision**, **Consequences** and
**Supersedes** in that order, written from the
[decision entry template](/documentation/standards/templates/decisions_entry.md) and added at the end of
the file of its subject. The approved program plan's ids stay in the heading in brackets:

| Subject | Plan decisions |
|---|---|
| `workspace.md` | D1, D5, D6, D7, D8, D11, D12, D13, D14, D15, D16, D17, D18 |
| `game_link.md` | D2, D22, plus the TCP link, the ports and the focus frame |
| `audio.md` | D19, D20, D21 |
| `bug_policy.md` | D3, D4, D9, D10 |

An entry is never reworded. When a decision changes, a new entry records the new one and names the old
one under Supersedes, so the latest entry on a subject is the one that holds. A link to an entry names
its file and the heading's anchor.

## Code

- The whole workspace follows these entries; the [crate catalogue](/documentation/program/crate_catalogue.md)
  names the crate each one shapes.

## Boundaries

- Depends on: the approved program plan, whose binding decisions the entries record; the behaviour specs
  in `documentation/program/behaviour_specs/`, which give their context.
- Used by: [CLAUDE.md](/CLAUDE.md), the [standards](/documentation/standards/README.md), the program
  records and the architecture documents, which cite the entries they follow.
- Rules: entries are never reworded and every heading keeps the `### YYYY-MM-DD — <decision>` form
  (review); every file opens with its status line (`cargo xtask verify status-lines`) and stays within
  500 lines (`cargo xtask verify markdown-placement`).

## Related documentation

- [Decision entry template](/documentation/standards/templates/decisions_entry.md) — the entry's parts
  and how to link one.
- [Program plan](/documentation/program/program_plan.md) — the binding decisions table and the program
  around it.
