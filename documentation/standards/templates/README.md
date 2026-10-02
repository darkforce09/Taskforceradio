**Status:** live

# Documentation templates

Copyable skeletons for every README kind of the
[README standard](/documentation/standards/readme_standard.md), for the document types under
`documentation/` (decision entries, runbooks, research snapshots, agent briefs) and for the four-point
module header. Each template carries a short worked sample written for a TFAR crate or document.

## Contents

```text
documentation/standards/templates/
├── agent_brief.md                  template for a per-agent brief; sample: the game link protocol slice
├── decisions_entry.md              template for a decision log entry; sample: the TCP game link
├── module_header.md                template for the four-point //! header; sample: the frame codec
├── readme_app.md                   README template for an app; sample: the 1.0 extension
├── readme_area_root.md             README template for an area root; sample: the audio crates
├── readme_command_line.md          README template for a command-line folder; sample: xtask verify
├── readme_crate_root.md            README template for a crate root; sample: the sample buffer crate
├── readme_data.md                  README template for a data folder; sample: the HRIR sphere
├── readme_documentation_folder.md  README template for a documentation folder; sample: the decisions
├── readme_domain.md                README template for a domain or subsystem; sample: the 1.0 routing src
├── readme_leaf.md                  README template for a leaf folder; sample: the protocol's frame kinds
├── research_snapshot.md            template for a frozen research snapshot; sample: the library survey
└── runbook.md                      template for a runbook; sample: probing an extension under wine
```

## How it works

Each template opens with its status line, its title and a **When to use** line naming the folders or
documents it fits. Then comes the skeleton: the sections in order inside a fenced `markdown` block, every
placeholder written as `<…>` and saying what goes there. A worked sample follows in a second fenced
block. The samples describe crates and documents of the target layout; while a crate is still a
skeleton, its sample shows the shape its README takes once the code is written, and the folder's own
README.md is written from the code and may differ.

The README templates are named `readme_<kind>.md` after the kind table in the README standard, and each
adds the kind sections that table lists. The document templates follow the section orders the
[documentation standards](/documentation/standards/documentation_standards.md) fix:

| Template | Sections, in order |
|---|---|
| `decisions_entry.md` | a dated `###` heading, then Context, Decision, Consequences, Supersedes |
| `runbook.md` | Prerequisites, numbered Steps with an **Expected:** line each, Verify, Troubleshooting, Related documentation |
| `research_snapshot.md` | one numbered section per need with its options and a Recommendation, then Recommended stack, Hard gaps, Sources |
| `agent_brief.md` | Goal, Steps, Owned files, Checks, Report additions, then the Budget line |
| `module_header.md` | Role, Position, Signals and state, Invariants |

The skeletons and samples sit in fences, so no gate reads them as documents: `readme-coverage` and
`readme-sections` judge only files named README.md, and `link-check` reads neither the links nor the
backticked paths inside a fence. A writer picks the kind or the document type, copies the skeleton, fills
every placeholder from the code, and runs the gates the standard names.

## Code

- The `cargo xtask new-crate` command (tools/xtask) writes a new crate's README.md and src/README.md
  from the crate root and domain skeletons.

## Boundaries

- Depends on: the README standard, which defines the README core, the kinds and their sections, and the
  documentation standards, which fix the document types' section orders, status lines and lifecycle.
- Used by: everyone who writes or reviews a README in the code trees or a document under
  `documentation/`; `cargo xtask new-crate`; the orchestrator, which writes every agent brief from
  `agent_brief.md`.
- Rules: one template per README kind, named as the kind table names it, and one per document type; each
  holds a status line, a **When to use** line, a skeleton and one worked sample, both fenced; no template
  passes 500 lines (`cargo xtask verify markdown-placement`).

## Related documentation

- [README standard](/documentation/standards/readme_standard.md) — the README core, the Contents
  grammar, the kinds and the gates.
- [Documentation standards](/documentation/standards/documentation_standards.md) — where documents live,
  their status lines, section orders and lifecycle.
