**Status:** live

# Documentation

Every document of Taskforceradio: the roadmap, the decisions, the architecture of the workspace, the game
link and the audio pipeline, the standards that shape the code and these pages, the program records the
milestones run from, and the runbooks. Start here to find the document on a subject and to learn which
source wins when two disagree.

## Contents

```text
documentation/
├── architecture/  how the port is built: workspace layout, game link protocol, audio pipeline
├── decisions/     the dated decision log, one file per subject
├── glossary.md    TFAR, TeamSpeak, audio and project terms, one heading each
├── program/       program records: plan, catalogue, laws and gates, bug fixes, progress, specs, briefs
├── roadmap.md     milestones M0 to M8 with checklists, acceptance tests and open questions
├── runbooks/      procedures: manual install and test, continuing in a new session
└── standards/     coding, documentation, README and commit rules, and the templates
```

## How it works

Documents live only in this folder. Each code folder (under crates/, tools/ and assets/) carries
its own README.md saying what the folder holds and where it stops, written to the
[README standard](/documentation/standards/readme_standard.md); the repository root holds `README.md`,
`CLAUDE.md` and its symlink `AGENTS.md`, and `reference/` holds its one README. A document about one part
of the code sits at the code's path with crates/ and src/ left out, so a document about the
1.0 radio routing crate goes in a folder named after the crate here; such folders are created with their
first document.

Every document opens with a status line. A **live** document tracks the code and changes in the same
commit as the code it describes. A **frozen record** keeps its words: the behaviour specs and research
under `documentation/program/` are frozen records of the original C++ as it was read, and new facts go
into a new record. An **archived** document records history and is never current. The
[documentation standards](/documentation/standards/documentation_standards.md) set the rules, and the
`cargo xtask verify` documentation gates check the ones a program can check.

### Reading order

1. [CLAUDE.md](/CLAUDE.md): the laws, the atlas, the environment, the commands.
2. [Progress](/documentation/program/progress.md): where the work stands and the next action.
3. [Roadmap](/documentation/roadmap.md): what each milestone delivers and how it is accepted.
4. The [program folder](/documentation/program/README.md) in its own resume order: progress, plan,
   catalogue, laws.
5. The document for the task at hand, from the table in [CLAUDE.md](/CLAUDE.md#5-where-to-look).

### Authority ladder

When two sources disagree, the higher one wins, and the lower one is fixed in the same commit:

```text
1. running code        what the built crates do, as their tests show
2. CLAUDE.md           the project laws
3. decisions/          the dated decision log
4. standards/          coding, documentation, README and commit rules, and this README
5. live documents      roadmap, architecture, program records, runbooks, glossary, code READMEs
6. frozen records      behaviour specs, research snapshots, executed agent briefs
```

The original C++ in `reference/` is the specification of behaviour, not of design: where a decision fixes
a bug (the [bug policy](/documentation/decisions/bug_policy.md)), the decision wins over the original.

## Code

- [Reference trees](/reference/README.md) — the two original mods the behaviour specs describe.
- The workspace (crates/, tools/): described by the
  [workspace layout](/documentation/architecture/workspace_layout.md) and the
  [crate catalogue](/documentation/program/crate_catalogue.md).

## Boundaries

- Depends on: the [standards](/documentation/standards/README.md) and their templates, which every
  document follows; the original mods in `reference/`, which the behaviour specs read.
- Used by: [CLAUDE.md](/CLAUDE.md), the root README, every code README that links a document, and every
  agent brief, which names the documents its agent reads first.
- Rules: every document under this folder opens with its status line (`cargo xtask verify status-lines`);
  live documents stay within 500 lines and only README.md files live outside this folder
  (`cargo xtask verify markdown-placement`); every link starts at the repository root and resolves
  (`cargo xtask verify link-check`); every folder has a README whose Contents block matches it
  (`cargo xtask verify readme-coverage`).

## Related documentation

- [Documentation standards](/documentation/standards/documentation_standards.md) — where documents live,
  status lines, lifecycle, writing rules.
- [README standard](/documentation/standards/readme_standard.md) — the README core and its kinds.
- [Glossary](/documentation/glossary.md) — the terms these documents use.
