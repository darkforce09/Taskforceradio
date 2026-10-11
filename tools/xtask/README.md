# Xtask

The repository's task runner, run as `cargo xtask` (the alias is in `.cargo/config.toml`): the
verify gates, the `ci-local` lane that is the project's CI, the release copy into the dist folder,
and the reference manifest. No task needs a shell script.

## How it works

`main.rs` parses the command line (`cli.rs`) and hands the `Repository` of this workspace to one
command under `commands/`. Every gate reads the laws from one module, `layout.rs`, which mirrors the
[crate boundary rules](/documentation/standards/crate_boundary_rules.md), and reads the repository
only through `support/`: the file list is what `git ls-files --cached --others --exclude-standard`
prints, so untracked new files are checked before their first commit and ignored build output never
is. Because gates never touch the file system directly, each gate's tests point it at a fixture tree
(`tests/fixture_tree.rs`) under `target/tmp/` and show it holding there and failing on a perturbed
copy.

A gate is one function, `check(&Repository) -> Result<Vec<Violation>>`: an `Err` means it could not
run, an empty list that it held. The runner prints each violation as `path:line: gate: message` and
exits 0 held, 1 violation, 2 could not run; a gate that could not run never reports success. The
crate gates read members through `cargo metadata --no-deps`; the source gates read Rust through a
code-only view, so comments and strings never match.

## Commands

| Command | Does |
|---|---|
| `cargo xtask verify <gate>` / `all` | Runs one gate or all of them; exits with the worst verdict |
| `cargo xtask ci ci-local` | Runs fmt, clippy (Linux), clippy (Windows extension side), `cargo test --workspace`, `cargo deny check`, `typos`, then every gate, each as its own child process with its log in `.workstation/logs/ci/<nn>-<step>.log`; stops at the first step that does not hold. Exit 0 all held, 1 a step failed, 2 a step could not run (a missing tool or a gate exiting 2) |
| `cargo xtask dist` | Builds both plugins (Linux) and both extensions (`x86_64-pc-windows-gnu`) in release and copies into `target/dist/tfar_0_9/` and `target/dist/tfar_1_0/` the plugin as `task_force_radio_linux_amd64.so` (0.9) or `TFAR_linux_amd64.so` (1.0), the extension as `task_force_radio_pipe_x64.dll`, the version's `radio-sounds/` from `reference/`, and `tfar_hrir/` with the HRIR sphere and its credits. TeamSpeak names a plugin after its file and the original receivers filter plugin commands by that name, so the `.so` names are the originals' |
| `cargo xtask reference-manifest [--force --reason <why>]` | Hashes every file under `reference/` except its README, the manifest and the 1.0 tree's `.git/`, and writes the sorted list to `reference/MANIFEST.sha256`; overwrites only with `--force` and a reason |

## Gates

| Gate | Checks |
|---|---|
| `crate-tiers` | Every manifest is a member with layout metadata; path = category + name; declared tier = computed tier; the category edge matrix, the version ban and the deliverable rules |
| `dependency-firewalls` | Each firewalled external crate only in its allowed crates; `tracy-client` optional |
| `unsafe-allowlist` | `#![allow(unsafe_code)]` only in the allow-listed crates, each with a reason comment; a `// SAFETY:` comment above every `unsafe` block |
| `file-length` | Production `.rs` files ≤ 500 lines, test files ≤ 1,000 |
| `test-placement` | No inline test module or `#[test]` in production files; test modules attach with `#[path = "tests/<file>.rs"]` |
| `history-residue` | No bug-catalogue, decision or milestone ids (`F-` or `OF-` with three digits, `D` with one or two, `M` with one) in `.rs` files or READMEs under `crates/` and `tools/` |
| `link-check` | Root-relative links, anchors and backticked repository paths in Markdown resolve; no `../` links |
| `editorconfig` | Final newline, LF, no trailing whitespace outside Markdown, indentation, on tracked text files outside `reference/` |
| `language-bans` | No Python, shell, Node or Makefiles; C/C++ only in the golden-vector oracle and shim, `.h` also in the TeamSpeak SDK headers |
| `reference-untouched` | Every file under `reference/` matches `reference/MANIFEST.sha256` (added, removed and changed files reported) |

## Boundaries

- Depends on no workspace crate; `clap`, `xshell`, `cargo_metadata`, `toml_edit`, `sha2`, `walkdir`
  and `ignore` stay here (`cargo xtask verify dependency-firewalls`).
- Used by developers and agents; `ci ci-local` is the CI. Gate and command names are cited by
  CLAUDE.md and the standards; renaming one changes them in the same commit.
