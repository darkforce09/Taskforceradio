**Status:** live

# Research behind the program plan

Frozen surveys that the binding decisions rest on: which conventions were adopted from which
projects, and which libraries were chosen or rejected for each need.

## Contents

```text
documentation/program/research/
├── conventions_survey.md   conventions adopted from TBD-Reforger, TBD-subtitles, Zed and rerun
└── library_survey.md       every need checked on crates.io: chosen crates, rejected crates, our own code
```

## How it works

Each survey is a frozen record dated 2026-10-02. A later survey is a new file next to these, and a
changed choice becomes a decision in the program plan; the surveys themselves are never edited.

## Code

- The workspace configuration and crates these choices shape (planned: the root Cargo.toml and the
  crates/ tree).

## Boundaries

- Depends on: the original projects' files named in each survey, and crates.io as of 2026-10-02.
- Used by: the program plan (D8, D18–D22), the standards and the crate catalogue.
- Rules: frozen; new research goes into a new dated file.

## Related documentation

- [Program plan](/documentation/program/program_plan.md) — the decisions these surveys support.
