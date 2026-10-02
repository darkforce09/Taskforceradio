**Status:** live

# README template: documentation folder

**When to use:** any folder under `documentation/`, the root included: a topic folder such as
`documentation/decisions/` or `documentation/runbooks/`, a folder of program records, a folder that
mirrors a code folder, or a long document split into a folder. The
[README standard](/documentation/standards/readme_standard.md) defines every rule this template follows;
the documentation folder kind adds Code.

## Skeleton

Copy the block and replace every `<…>` placeholder; each one says what goes there. The status line
`**Status:** live` comes first, as it does for every README under `documentation/`, the index of a folder
of frozen records included.

````markdown
**Status:** live

# <What the documents cover, in plain words: no path, no backticks>

<One to three sentences: what the folder documents and who reads it.>

## Contents

```text
<repository path of the folder>/
├── <child folder>/   <what it covers: a lowercase phrase, no closing period>
├── <document>.md     <what it covers>
└── <document glob>   <the collection, when the documents are alike>
```

## How it works

<How the documents are organised: which template each follows, how they are named, which one to read
first, and how a new one is added. Leave the section out when the folder has no child folders and at
most three files.>

## Code

- [<code folder name>](/<repository path of a code folder>/) — <what the documents say about it>

## Boundaries

- Depends on: <the templates and standards the documents follow, and the sources they draw on>
- Used by: <the documents, READMEs and code comments that link here>
- Rules: <the invariants a change must keep: naming, status lines, what may change and what is frozen,
  and the gate that checks each>

## Related documentation

- [<document title>](/documentation/<path to the document>) — <what it covers>
````

## Worked sample

Written from `documentation/decisions/`, shortened.

````markdown
**Status:** live

# Decisions

The log of choices that shape the port and would otherwise be argued again: the workspace and its
conventions, the game link, the audio libraries, and the bug policy.

## Contents

```text
documentation/decisions/
├── audio.md       HRTF through the hrtf crate, the distance and cone gain model, the filter libraries
├── bug_policy.md  fix every bug, keep every feature, C++ only as an oracle, the port order
├── game_link.md   Proton only, the TCP game link, its ports, the focus frame, the TeamSpeak bindings
└── workspace.md   the repository, names, layout, conventions, git, panics, dist, toolchain, libraries
```

## How it works

One `###` entry per decision, `<date> — <decision>`, with Context, Decision, Consequences and Supersedes,
added at the end of its subject's file from the decision entry template. A changed decision gets a new
entry naming the old one.

## Code

- The workspace as a whole: every crate follows the workspace decisions.

## Boundaries

- Depends on: the approved program plan, the behaviour specs.
- Used by: CLAUDE.md, the standards, the program records, the architecture documents.
- Rules: entries are never reworded (review); every file opens with its status line
  (`cargo xtask verify status-lines`).
````
