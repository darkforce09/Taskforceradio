**Status:** live

# README standard

Every folder in the code trees and in `documentation/` carries a README.md built the same way: a fixed
core of sections, the sections its kind adds, and a Contents block that a gate checks against the folder.
The README tells people and agents what a folder holds, how it works and where it stops, and links the
documents that go deeper.

## Which folders carry a README

The README span is every tracked folder at or under crates/, tools/, assets/ and `documentation/`, the
roots included, minus the exempt folders and everything below them:

- a folder named `tests` (sibling unit tests, integration tests and their fixtures);
- a folder named `target` (build output);
- a folder whose name begins with `.` (tool configuration).

A crate's `src/` folder and every module folder inside it are in the span. An exempt folder needs no
README, and its parent's Contents block describes it in one line. The repository root's `README.md` and
`reference/README.md` lie outside the span; they follow the core where it fits but no gate reads their
sections. A folder exists when it holds a tracked file.

## The README core

Every README holds these sections, in this order. A README under `documentation/` starts with the status
line `**Status:** live` and a blank line above its title; a README in a code tree carries no status line.

1. **Title.** An H1 with a human-readable name: no path, no backticks (`# Game link protocol`, never a
   folder name such as `tfar_game_link_protocol/`).
2. **Purpose.** One to three sentences under the title saying what the folder is for.
3. **`## Contents`.** The folder's direct children as a `text` tree, one line each with its role (see
   [The Contents block](#the-contents-block)).
4. **`## How it works`.** How the children work together: the flow of data and control through the
   folder, the main types, the threads involved, the invariants that span files, and an ASCII diagram in
   a `text` block when one helps. A folder with no child folders besides exempt ones and at most three
   files (README.md not counted) may leave this section out.
5. **The kind sections.** The sections the folder's kind adds (see [Kinds](#kinds)), in the order the
   kind table lists them.
6. **`## Boundaries`.** Exactly three top-level bullets, in this order and spelled this way:
   - `Depends on:` what the folder uses: modules, workspace crates, external crates, files and host
     programs (TeamSpeak, Arma), read from its imports and calls.
   - `Used by:` everything outside the folder that uses it; `nothing` when nothing does.
   - `Rules:` the invariants particular to this folder that a change must keep, each with the test or
     gate that holds it wherever one does; repository-wide laws are not restated.

   A bullet may run over several lines and may hold a nested list.
7. **`## Related documentation`.** Repository-root links to the documents that go deeper into this
   folder, each followed by a few words on what it covers. Left out when there are none.

Headings are unnumbered and spelled exactly as above. A README holds no other `##` heading; finer
structure goes into `###` headings inside How it works or a kind section. A kind section with nothing to
hold keeps its heading and says so in one line (`None: the crate reads no setting.`).

## The Contents block

`cargo xtask verify readme-coverage` reads the Contents block and checks it against the folder; its code
is the final word.

The Contents block is the first fenced code block whose info string is exactly `text` and that opens
after the `## Contents` heading and before the next `## ` heading.

- **Root line:** line 1 of the block is exactly the folder's repository-relative path followed by `/`.
- **Entry lines:** one direct child each: an optional tree-drawing prefix (`├── `, `└── `), the name or a
  glob, two or more spaces, and a non-empty role. A folder entry ends in `/`; a file entry does not. A
  nested line (`│   ├── `) fails.
- **Globs:** `*`, `?`, `[…]` and `{a,b}` match whole names.
- **Matching:** every tracked direct child except `README.md` matches exactly one entry of its own kind,
  and every entry matches at least one tracked child.

### Writing the block

- Draw the tree: `├── ` before each entry and `└── ` before the last. Entries run in case-insensitive name
  order, a folder compared without its `/`, so `Cargo.toml` comes before `src/` and `error.rs` before
  `frame_kinds/`. Roles line up in one column.
- A role says what the child is for, not what it is made of: a lowercase phrase with no closing period,
  the line within about 100 characters.
- List dot-files. `tests/` gets one line; its insides are exempt.
- A folder that holds only its README.md has a block with the root line alone.
- Keep other `text` blocks out of the Contents section.

```text
crates/game_link/tfar_game_link_protocol/src/
├── error.rs      the protocol's error type: short frame, unknown kind, oversize payload, bad magic
├── frame.rs      the length-prefixed frame header and its encoder and decoder
├── hello.rs      the HELLO handshake: magic, protocol number, mod flavour
├── lib.rs        the crate header, the module tree and the re-exports
├── prelude.rs    the types every user of the protocol imports
└── tests/        unit tests for the frame codec and the handshake
```

## Kinds

A folder's kind decides which sections follow How it works. Each kind has a template in the
[templates folder](/documentation/standards/templates/README.md), with a skeleton and a worked sample.

| Kind | Kind sections, in order | Template | Examples |
|---|---|---|---|
| crate root | Getting started, Configuration, Public surface | `readme_crate_root.md` | a library under crates/, tools/xtask/, tools/tfar_ts3_mock/ |
| app | Getting started, Configuration, Exports | `readme_app.md` | crates/version_1_0/tfar_plugin_1_0/, crates/version_0_9/tfar_extension_0_9/ |
| area root | Getting started | `readme_area_root.md` | crates/, crates/audio/, crates/version_1_0/, tools/, assets/ |
| domain or subsystem | Public surface | `readme_domain.md` | a crate's src/, a module folder with child folders |
| leaf | none | `readme_leaf.md` | a module folder of source files only |
| command-line | Commands | `readme_command_line.md` | the xtask command folders, a mock's scenario runner |
| data | Format, Producers and consumers | `readme_data.md` | assets/hrir/, the golden-vector fixture folders |
| documentation folder | Code | `readme_documentation_folder.md` | `documentation/decisions/`, `documentation/standards/` |

The legal `##` headings, in their one order, are: Contents, How it works, Getting started, Configuration,
Public surface, Exports, Commands, Format, Producers and consumers, Code, Boundaries, Related
documentation. `cargo xtask verify readme-sections` holds that order and each kind's set.

### What each kind section holds

- **`## Getting started`:** the few commands, run from the repository root, that build, run or test what
  the folder holds, each with what to expect. Name the target triple when it is not the host's
  (`--target x86_64-pc-windows-gnu` for the extensions).
- **`## Configuration`:** every setting the folder's code reads (Cargo features, TeamSpeak settings, the
  1.0 plugin settings sent with SETCFG, files under TeamSpeak's config path, command-line switches such
  as `-tfdebug`), with its default and the file that reads it.
- **`## Public surface`:** what code outside the folder may use: the modules, types, functions and
  binaries that cross the folder's boundary, never an inventory of everything `pub`.
- **`## Exports`:** for an app, the C ABI symbols its host loads (`ts3plugin_*` for TeamSpeak,
  `RVExtension` and `RVExtensionVersion` for Arma), the file name `cargo xtask dist` gives the artifact,
  and where the owner installs it.
- **`## Commands`:** each command the folder defines: synopsis, what it does, exit codes, one example.
- **`## Format`:** the file format: encoding, layout, naming, and how to add a file.
- **`## Producers and consumers`:** what writes the files and what reads them, with paths.
- **`## Code`:** the code folders the documents describe, as repository-root links.

### Picking a kind

Take the first that fits: documentation folder (anything under `documentation/`); app (the folder
of one of the four deliverables, `tfar_plugin_0_9`, `tfar_extension_0_9`, `tfar_plugin_1_0` and
`tfar_extension_1_0`, each in its version category under crates/); area root (crates/, a category
folder under crates/, tools/, assets/); crate root (any other folder holding a `Cargo.toml`); data;
command-line (a folder that defines a binary's subcommands); domain or subsystem (any other folder
with child folders besides exempt ones, a crate's `src/` included); leaf (any other folder).

## Writing rules

- **Truth.** Every claim is checked against the code it describes. When a README and the code disagree,
  the code wins and the README is fixed in the same commit.
- **Present tense.** A README says what the folder is and does. No history and no plans; the roadmap owns
  plans, git owns history. A skeleton module says what it is for and that its code is not written yet,
  nothing more.
- **No tracking ids.** A code README never names a milestone, a finding id or an agent; the roadmap and
  the program records do.
- **Links.** Repository-root links, never `../` climbs.
- **Paths.** A backticked path to anything outside the README's folder is a full repository path; inside
  the folder, a path relative to it. A path that does not exist yet is written as plain text.
  `cargo xtask verify link-check` verifies every backticked full repository path.
- **Original code.** A README that ports original behaviour names the source as a `reference/` path with
  a line number (`reference/tfar_1_0/ts/src/RadioEffect.hpp`), never as "the old code".
- **Terminology.** A README links a term's first use to the [glossary](/documentation/glossary.md) when
  the glossary holds it.
- **Parents and children.** A parent summarises each child in its one Contents line and, where it helps,
  one clause in How it works. It never inventories a child's files.
- **README or document.** The README describes its own folder at a high level. Protocols, pipelines,
  decisions and behaviour specs live under `documentation/` and are linked under Related documentation.
- **Same commit.** A code change updates, in the same commit, the README of every folder whose contents,
  surface, commands or boundaries it changes.
- **Diagrams.** ASCII, in `text` blocks.
- **Length.** A leaf runs about 10 to 60 lines, a domain 40 to 200, a root up to 400; no README passes 500
  lines.

## Gates

| Gate | What it checks |
|---|---|
| `cargo xtask verify readme-coverage` | every folder in the span has a tracked README.md whose Contents block matches the folder |
| `cargo xtask verify readme-sections` | the title, the `##` headings in their one order and within the kind's set, Contents and Boundaries present, Boundaries' three bullets, no status line in a code README |
| `cargo xtask verify status-lines` | every document under `documentation/` opens with its status line |
| `cargo xtask verify markdown-placement` | the code trees hold no Markdown but README.md; live documents stay within 500 lines |
| `cargo xtask verify link-check` | every link reaches a tracked target and anchor; every backticked repository path exists; no `../` links |

## Related documentation

- [Documentation standards](/documentation/standards/documentation_standards.md) — where documents live,
  names, status lines, lifecycle.
- [Templates](/documentation/standards/templates/README.md) — one skeleton per README kind and document
  type.
