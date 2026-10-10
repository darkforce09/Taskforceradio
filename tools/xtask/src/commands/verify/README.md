# Verify gates

The `cargo xtask verify` gates: each checks one law of the repository and says held, violation or
could not run. Every agent runs the documentation gates and its slice's structural gate before its
report; `cargo xtask ci ci-local` runs them all.

## Contents

```text
tools/xtask/src/commands/verify/
├── crate_anatomy.rs         lib.rs shape, prelude, thiserror error, no anyhow, inherited fields, features
├── crate_tiers.rs           membership, layout metadata, tiers, the edge matrix, the version ban
├── dependency_firewalls.rs  each firewalled external crate only in its allowed crates
├── editorconfig.rs          UTF-8, LF, final newline, trailing whitespace, space indentation
├── file_length.rs           the 500 and 1000 line ceilings
├── gate.rs                  the gate table, the gate names and the violation type
├── gate_runner.rs           `verify <gate>` and `verify all`: output lines and the exit code
├── language_bans.rs         no scripts; C and C++ only in the golden-vector oracle and shim
├── link_check.rs            links, anchors and backticked repository paths exist; no `../`
├── markdown_placement.rs    Markdown only where it belongs; live documents within 500 lines
├── mod.rs                   the module list
├── module_headers.rs        the four-point `//!` header on crate roots and files of 80+ lines
├── readme_coverage.rs       a README in every span folder, its Contents listing the folder exactly
├── readme_sections.rs       title, sections per kind in the standard's order, Boundaries bullets
├── reference_untouched.rs   every file under `reference/` against `reference/MANIFEST.sha256`
├── source_files.rs          the Rust sources the source gates walk and the crate each belongs to
├── status_lines.rs          the status line of every document; none in code READMEs
├── test_placement.rs        no inline test modules or test functions in production files
├── unsafe_allowlist.rs      where `unsafe_code` may be lifted; `// SAFETY:` above unsafe blocks
└── tests/                   one fixture test file per gate, clean and perturbed, and the runner's
```

## How it works

`gate_runner.rs` runs the selected gates through the table in `gate.rs`. A gate is one function,
`check(&Repository) -> Result<Vec<Violation>>`: an `Err` means it could not run, an empty list that
it held. The runner prints each violation on stdout as `path:line: gate: message` and one verdict
line per gate on stderr, and exits with the worst verdict. The crate gates read members through
`cargo metadata --no-deps`; the source gates read Rust through the code-only view, so comments and
strings never match; the documentation gates read Markdown outside code fences only.
`readme-sections` reads the kind table and the heading order from
`documentation/standards/readme_standard.md` at run time, so the standard stays the one source.

```text
verify <gate>|all ──► gate_runner ──► Gate::check ──► <gate>.rs ──► layout.rs, support/
                           │
                           └── stdout: path:line: gate: message    stderr: gate: held | N violation(s)
```

## Commands

All run as `cargo xtask verify <gate>` from the repository root. Exit code 0 means held, 1 a
violation, 2 could not run; a gate that could not run never reports success.

### all

- Synopsis: `cargo xtask verify all`
- Does: runs every gate below in this order and prints all their violations.
- Exit codes: the worst of the gates' codes.
- Example: `cargo xtask verify all`

### crate-tiers

- Synopsis: `cargo xtask verify crate-tiers`
- Does: every manifest under crates and tools is a member; each member's layout table has
  `category`, `tier` and `targets`; path = category + name; name = folder name; declared tier =
  1 plus the highest normal or build workspace dependency's tier (0 with none); every edge keeps
  the category edge matrix, the version ban (dev-dependencies included), the extension-app and
  plugin-app rules (keyed by crate name: the four deliverables sit in their version folders),
  tools never on a deliverable, and dev-dependencies never on a deliverable.
- Exit codes: 0 held, 1 violation, 2 `cargo metadata` failed.
- Example: `cargo xtask verify crate-tiers`

### dependency-firewalls

- Synopsis: `cargo xtask verify dependency-firewalls`
- Does: each firewalled external crate is a direct dependency only of its allowed crates;
  `tracy-client` is optional.
- Exit codes: 0 held, 1 violation, 2 `cargo metadata` failed.
- Example: `cargo xtask verify dependency-firewalls`

### crate-anatomy

- Synopsis: `cargo xtask verify crate-anatomy`
- Does: `[lints] workspace = true`; edition, rust-version, license and publish from the workspace;
  every dependency `workspace = true`; only `test_fixtures`, and `tracy` and `api_profiler` in the
  plugin apps, as features; for a library, `lib.rs` within 80 lines holding only the header,
  attributes, `mod`, `pub mod` and `pub use` (never of a workspace crate), `prelude.rs`, `error.rs`
  with `thiserror`, and no `anyhow`.
- Exit codes: 0 held, 1 violation, 2 a manifest does not parse.
- Example: `cargo xtask verify crate-anatomy`

### unsafe-allowlist

- Synopsis: `cargo xtask verify unsafe-allowlist`
- Does: `allow(unsafe_code)` only as `#![allow(unsafe_code)]` at the crate root of the allow-listed
  crates, with a `//` reason comment on the line directly above; every `unsafe {` with a
  `// SAFETY:` comment in the comment lines directly above it.
- Exit codes: 0 held, 1 violation, 2 a source is unreadable or no source root exists.
- Example: `cargo xtask verify unsafe-allowlist`

### file-length

- Synopsis: `cargo xtask verify file-length`
- Does: production `.rs` files within 500 lines, files under a `tests` folder within 1000.
- Exit codes: 0 held, 1 violation, 2 no source root exists.
- Example: `cargo xtask verify file-length`

### test-placement

- Synopsis: `cargo xtask verify test-placement`
- Does: no inline test module, no `#[test]` function, and no test module without
  `#[path = "tests/<file>.rs"]` in a production file.
- Exit codes: 0 held, 1 violation, 2 no source root exists.
- Example: `cargo xtask verify test-placement`

### module-headers

- Synopsis: `cargo xtask verify module-headers`
- Does: crate roots (`src/lib.rs`, `src/main.rs`) and production files of 80 lines or more open
  with a `//!` summary and the Role, Position, Signals and state, and Invariants paragraphs, in
  order.
- Exit codes: 0 held, 1 violation, 2 no source root exists.
- Example: `cargo xtask verify module-headers`

### readme-coverage

- Synopsis: `cargo xtask verify readme-coverage`
- Does: every folder under crates, tools, documentation and assets that holds a listed file
  (except `tests`, `target` and dot-folders) has a README.md whose Contents block lists exactly its
  direct children.
- Exit codes: 0 held, 1 violation, 2 `git ls-files` failed.
- Example: `cargo xtask verify readme-coverage`

### readme-sections

- Synopsis: `cargo xtask verify readme-sections`
- Does: each README has an H1 title without backticks or a path, only legal `##` headings in the
  standard's order and within its kind's set, Contents, How it works (unless the folder is small),
  its kind's sections and Boundaries with the three bullets; a code README has no status line.
- Exit codes: 0 held, 1 violation, 2 the README standard or its kind table is missing.
- Example: `cargo xtask verify readme-sections`

### status-lines

- Synopsis: `cargo xtask verify status-lines`
- Does: every Markdown file under `documentation/` opens with `**Status:** live`,
  `**Status:** frozen record (YYYY-MM-DD)` or `**Status:** archived`; no other Markdown file does.
- Exit codes: 0 held, 1 violation, 2 `git ls-files` failed.
- Example: `cargo xtask verify status-lines`

### markdown-placement

- Synopsis: `cargo xtask verify markdown-placement`
- Does: Markdown outside `reference/` lives under `documentation/`, as a README.md, as the root
  CLAUDE.md, AGENTS.md or README.md, or as a CREDITS.md under `assets/`; a live document stays
  within 500 lines.
- Exit codes: 0 held, 1 violation, 2 `git ls-files` failed.
- Example: `cargo xtask verify markdown-placement`

### link-check

- Synopsis: `cargo xtask verify link-check`
- Does: every link that is not a URL resolves, with its anchor; no link climbs with `../`; every
  backticked string whose first segment is crates, tools, documentation, assets or reference
  names something that exists (a `:line` suffix and a `#anchor` are dropped first). Bare file names
  and crate-relative paths are not paths here.
- Exit codes: 0 held, 1 violation, 2 `git ls-files` failed.
- Example: `cargo xtask verify link-check`

### editorconfig

- Synopsis: `cargo xtask verify editorconfig`
- Does: on every listed text file outside `reference/`: UTF-8, LF, a final newline, no trailing
  whitespace outside Markdown, no tab indentation in Rust, TOML, JSON and YAML.
- Exit codes: 0 held, 1 violation, 2 `git ls-files` failed or a file is unreadable.
- Example: `cargo xtask verify editorconfig`

### language-bans

- Synopsis: `cargo xtask verify language-bans`
- Does: no `.py`, `.sh`, `.bash`, `.js`, `.ts` or Makefile outside `reference/`; C and C++ sources
  and headers only under `tools/tfar_golden_vectors/` in its oracle and shim folders.
- Exit codes: 0 held, 1 violation, 2 `git ls-files` failed.
- Example: `cargo xtask verify language-bans`

### reference-untouched

- Synopsis: `cargo xtask verify reference-untouched`
- Does: hashes every file under `reference/` except its README, the manifest and the 1.0 tree's
  `.git/` folder and compares them with `reference/MANIFEST.sha256`, reporting each added,
  removed and changed file.
- Exit codes: 0 held, 1 a file differs, 2 the manifest is missing or unreadable.
- Example: `cargo xtask verify reference-untouched`

## Boundaries

- Depends on: `layout.rs` for every law; `support/` for the listing, the members, the Rust and
  Markdown readers; `cargo_metadata`, `toml_edit`, `sha2`, `walkdir`; at run time git and cargo.
- Used by: `main.rs`, the `ci-local` lane (one child process per gate), every agent's stage gate.
- Rules:
  - every gate has a fixture test that shows it holding on the sample workspace and failing on a
    perturbed copy (`tests/<gate>.rs`), and every gate holds on the untouched sample
    (`every_gate_holds_on_the_sample_workspace` in `tests/gate_runner.rs`);
  - the gate names are stable: CLAUDE.md, the standards and the program records cite them.

## Related documentation

- [Laws and gates](/documentation/program/laws_and_gates.md) — what each gate holds and why.
- [README standard](/documentation/standards/readme_standard.md) — the span, the Contents grammar
  and the kind table the README gates read.
