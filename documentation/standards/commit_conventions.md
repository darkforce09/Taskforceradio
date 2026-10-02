**Status:** live

# Commit conventions

How changes are committed: the message format, what belongs in one commit, who commits, and the checks to
run first (decision D11).

## Message

Conventional Commits: `type(scope): lowercase summary`, at most 72 characters, no trailing period.

- **Types:** `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`, `chore`.
- **Scopes:** the crate touched, its package name in kebab-case (`tfar-ts3-host`,
  `tfar-game-link-protocol`, `tfar-plugin-1-0`, `tfar-voice-pipeline-0-9`, `xtask`), or one of:
  - `workspace`: root configuration, several crates at once, the reference archive;
  - `program`: the program records under `documentation/program/` and the roadmap;
  - `docs`: any other document under `documentation/`, `CLAUDE.md`, the root README.

  Omit the scope only when a change truly spans everything.
- **Body:** two to four wrapped lines on what changed and why. Name the bug-fix id (F-NNN) or decision id
  (D12) the change carries out; the message is the one place outside the program records where these ids
  belong.
- **Trailer:** an AI-authored commit ends with a `Co-Authored-By:` line naming the model.

```text
fix(tfar-game-commands-1-0): check the token count of tangent messages

TANGENT with an optional classname read its fields by fixed index, so an
eight-token message shifted every field. Fields are now read by position
with a count check, and a short message is an error value (F-NNN).

Co-Authored-By: <model name> <noreply@anthropic.com>
```

## What goes in one commit

- One logical change, with the documentation it affects (comments, READMEs, documents) and the roadmap
  boxes it ticks.
- During a program, one commit per slice: the slice's code, its READMEs and its program-record updates.
- Stage files with explicit paths; never `git add -A` or `git add .`.
- Never commit build output (the target folders), built DLLs or `.so` files, or machine-local
  settings (see `.gitignore`). `reference/tfar_1_0/` is gitignored and guarded by its manifest instead.

## Who commits

- Commits go straight to `main`. There are no feature branches and no remote yet.
- During a program, only the orchestrator commits. Agents never run `git add`, `git commit`, `git stash`,
  `git reset` or `git checkout`; they report their files and the orchestrator reviews, gates and
  commits.

## Before committing

These pass, each run alone from the repository root:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p tfar_extension_1_0 -p tfar_extension_0_9 --target x86_64-pc-windows-gnu -- -D warnings
cargo test --workspace
cargo deny check
typos
cargo xtask verify all
```

A slice commit may run the fast stage gate instead (fmt, clippy, the tests of the touched crates and the
documentation gates); the milestone close runs `cargo xtask ci ci-local`, which runs all of them in order.

## Related documentation

- [Coding standards](/documentation/standards/coding_standards.md) — the rules the checks hold.
- [Laws and gates](/documentation/program/laws_and_gates.md) — the stage gate and the milestone gate.
