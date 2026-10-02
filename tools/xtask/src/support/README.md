# Xtask support

The readers every xtask command shares: the repository listing, the workspace members, a code-only
view of Rust sources, a small Markdown reader, Contents globs, the README span and the reference
manifest format.

## Contents

```text
tools/xtask/src/support/
├── cargo_workspace.rs   the workspace members from `cargo metadata`: folder, layout table, edges
├── glob_pattern.rs      whole-name globs for Contents entries: `*`, `?`, `[…]`, `{a,b}`
├── markdown.rs          fences, headings, links, inline code spans and GitHub heading anchors
├── mod.rs               the module list
├── readme_span.rs       which folders carry a README, a folder's children, the Contents grammar
├── reference_hashes.rs  the sha256 list of `reference/`: hashing, rendering and parsing
├── repository.rs        the repository root and its file list from `git ls-files`
├── rust_source.rs       Rust lines with comments and literal contents blanked
└── tests/               unit tests of each reader
```

## How it works

`repository.rs` runs `git ls-files --cached --others --exclude-standard` once per `Repository`
and caches the sorted list; every other reader works on that list or on one file's text. A
failed git call is an error, so a gate built on the list reports "could not run" instead of
holding on an empty tree. `cargo_workspace.rs` runs `cargo metadata --no-deps`, which needs no
network. `rust_source.rs` keeps every line and column of a source but blanks comments and the
insides of string and character literals, so a gate that looks for `mod tests {` or an unsafe
block never matches a comment or a string. `markdown.rs` skips everything inside code fences.

## Boundaries

- Depends on: `cargo_metadata`, `sha2`, `walkdir`, `xshell`, `anyhow`, and `layout.rs` for the
  span and the reference exclusions; at run time git and cargo.
- Used by: every module under `tools/xtask/src/commands/`.
- Rules:
  - paths are repository-relative with `/` separators (`support/tests/repository.rs`);
  - the code view has as many lines as the source (`support/tests/rust_source.rs`);
  - a manifest rendered and parsed back is unchanged (`support/tests/reference_hashes.rs`).
