**Status:** live

# Documentation standards

How documents in this repository are laid out, named, written and kept current. A rule held by a program
names the `cargo xtask verify` gate that holds it; the rest are held in review.

## Where documents live

- Every document lives under `documentation/`. Outside it, only `README.md` files (one per folder of the
  code trees, plus `reference/README.md`) and the root `README.md`, `CLAUDE.md` and `AGENTS.md` exist
  (`cargo xtask verify markdown-placement`). Markdown inside `reference/` belongs to the original mods and
  is ignored.
- A document about one part of the code sits at the code's path with crates/ and src/ left out:
  a document about the 1.0 voice pipeline goes in a folder named version_1_0/tfar_voice_pipeline_1_0/
  under `documentation/`, created with its first document.
- Topics that span the code have their own folders:

  | Folder | Holds |
  |---|---|
  | architecture/ | how the port is built: workspace layout, game link protocol, audio pipeline |
  | decisions/ | the dated decision log, one file per subject |
  | standards/ | these rules, the README standard, coding and commit rules, templates |
  | program/ | the program records: plan, crate catalogue, laws and gates, bug fixes, progress, behaviour specs, research, agent briefs |
  | runbooks/ | procedures someone runs step by step |

  `glossary.md` and `roadmap.md` sit at the top of `documentation/`.

## Files

- Names are snake_case `.md`, apart from `README.md`.
- The first line of every document under `documentation/` is its status line, one of
  `**Status:** live`, `**Status:** frozen record (YYYY-MM-DD)` or `**Status:** archived`
  (`cargo xtask verify status-lines`). README files outside `documentation/`, `CLAUDE.md` and the root
  `README.md` carry none.
- A live document stays within 500 lines; a longer one splits into a folder with a README index
  (`cargo xtask verify markdown-placement`).
- Live documents carry no dates, except decision entries, the program's execution log and the status
  line of frozen records.

## READMEs

Every folder in `documentation/` and in the code trees has a README.md
(`cargo xtask verify readme-coverage`), written to the
[README standard](/documentation/standards/readme_standard.md): a title, a purpose, a Contents block that
lists the folder exactly, How it works, the sections of the folder's kind, Boundaries with its three
bullets, and Related documentation (`cargo xtask verify readme-sections`).

## Templates

Every README kind and document type has a skeleton with a worked sample in
[templates](/documentation/standards/templates/README.md). A new README, decision entry, runbook,
research snapshot, agent brief or module header starts from its template.

## Decision entries

In the decision log (`documentation/decisions/`, one file per subject, each within 500 lines), one `###`
heading per decision: `### YYYY-MM-DD — <the decision>`, then **Context**, **Decision**,
**Consequences** and **Supersedes**, in that order. The approved plan's decision ids (D1 to D22) stay in
the entry's heading so the program records can cite them. A changed decision gets a new entry naming the
old one under Supersedes; old entries are never reworded.

## Runbooks

Prerequisites, then numbered steps (one command each, followed by an **Expected:** line), then Verify,
Troubleshooting and Related documentation. A step that runs in the game or TeamSpeak says so and says
what the owner sees; no step copies files into the game or TeamSpeak on the owner's behalf.

## Program records

The records under `documentation/program/` follow their own shapes, set by the program plan:

- `progress.md`: Current state, Milestones, Execution log, Amendments, Open findings, Handoff.
- `bug_fixes.md`: one F-id per original bug, with its `reference/` file and line, the original and the
  fixed behaviour, the owning crate, the milestone and the test.
- `agent_briefs/`: `shared_brief.md` plus one brief per slice, named m<N>_<agent id>_<role>.md, written
  from the [agent brief template](/documentation/standards/templates/agent_brief.md). A brief is live
  while its slice runs and becomes a frozen record when the slice is committed.
- `behaviour_specs/` and `research/`: frozen records of the original code and of the surveys.

Milestone and finding ids (M3, F-012) belong here and in the roadmap; code, code READMEs and live
architecture documents never carry them.

## Writing

- Plain words for a reader who has not seen the code. Define a term in the
  [glossary](/documentation/glossary.md) and link its first use.
- Present tense in live documents and READMEs: what the code does, not how it came to be.
- Links start at the repository root (`/documentation/glossary.md`), never `../`. Every link and anchor
  in a live document resolves (`cargo xtask verify link-check`).
- Every backticked repository path exists. A planned path (a crate not written yet, a file a later
  milestone adds) is written as plain text, not code, until it exists (`cargo xtask verify link-check`).
- A behaviour of the original mods is cited by `reference/` path and line
  (`reference/tfar_1_0/ts/src/SharedMemoryHandler.cpp:14`); the citation describes the original, never
  the port.
- Diagrams are ASCII inside `text` code blocks.

## Lifecycle

- **Live** documents track the code and change in the same commit as the code they describe.
- **Frozen records** (behaviour specs, research snapshots, executed agent briefs) keep their words; only
  broken links are fixed. New facts go into a new record.
- **Archived** documents record history and are never current.

## Authority

Highest first; the lower source is fixed when two disagree:

1. running code;
2. `CLAUDE.md`;
3. decisions;
4. the documentation README and these standards;
5. live documents;
6. frozen records.

## Related documentation

- [Documentation map](/documentation/README.md) — the tree, the reading order and the authority ladder.
- [README standard](/documentation/standards/readme_standard.md) — the README core, Contents grammar and
  folder kinds.
- [Templates](/documentation/standards/templates/README.md) — skeletons for every README kind and
  document type.
