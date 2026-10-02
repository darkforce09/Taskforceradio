**Status:** live

# README template: command-line

**When to use:** a folder that defines a binary's subcommands: xtask's command groups (verify, ci, dist,
new-crate, reference-manifest), the scenario runner of the TeamSpeak mock, the extension probe's call
list. The [README standard](/documentation/standards/readme_standard.md) defines every rule this
template follows; the command-line kind adds Commands.

## Skeleton

Copy the block and replace every `<…>` placeholder; each one says what goes there. Commands holds one
`###` heading per command the folder defines, with the same four bullets under each.

````markdown
# <Name of the command group, in plain words: no path, no backticks>

<One to three sentences: what the commands are for and who runs them.>

## Contents

```text
<repository path of the folder>/
├── <file>  <the command it defines, or the parsing it holds: a lowercase phrase, no closing period>
└── tests/  <what the tests cover>
```

## How it works

<How an invocation flows: where the arguments are parsed, where each command's work runs, what it reads
and writes, and how the exit code is formed. An ASCII diagram in a text block helps here.>

## Commands

<One line on how to run the commands, and what they share: the exit codes, the output format.>

### <command name>

- Synopsis: `<the usage line, as the command's own --help prints it>`
- Does: <what it does>
- Exit codes: <each code and what it means>
- Example: `<one real invocation, run from the repository root>`

## Boundaries

- Depends on: <the modules and crates the commands call, read from their imports>
- Used by: <the people and commands that run them, found with git grep>
- Rules: <the invariants a change here must keep: stable names, exit codes, and the test or gate that
  checks each>

## Related documentation

- [<document title>](/documentation/<path to the document>) — <what it covers>
````

## Worked sample

Written for xtask's verify command group, shortened to two of its gates, as it reads once its code is
written.

````markdown
# Verify gates

The `cargo xtask verify` gates: each checks one law of the repository and says held, violation or could
not run. The orchestrator runs them at every slice; `cargo xtask ci ci-local` runs them all.

## Contents

```text
tools/xtask/src/commands/verify/
├── crate_tiers.rs           tiers, categories, the edge matrix and the version ban
├── file_length.rs           the 500 and 1000 line ceilings
├── mod.rs                   the gate table, `verify <gate>` and `verify all`
├── reference_untouched.rs   every file under reference/ against MANIFEST.sha256
└── tests/                   one fixture tree per gate, clean and perturbed
```

## How it works

`mod.rs` parses the gate name and runs it. A gate returns a list of violations or an error saying why it
could not run; the command prints one `path:line: gate: message` line per violation and exits with the
gate's verdict. `verify all` runs every gate and exits with the worst verdict.

## Commands

All run as `cargo xtask verify <gate>` from the repository root. Exit code 0 means held, 1 a violation,
2 could not run; a gate that could not run never reports success.

### crate-tiers

- Synopsis: `cargo xtask verify crate-tiers`
- Does: reads `cargo metadata` and checks every member's layout metadata, tier and category edges.
- Exit codes: 0 held, 1 violation, 2 cargo metadata failed.
- Example: `cargo xtask verify crate-tiers`

### reference-untouched

- Synopsis: `cargo xtask verify reference-untouched`
- Does: hashes every file under reference/ except the README, the manifest and tfar_1_0/.git/ and
  compares them with the manifest, reporting added, removed and changed files.
- Exit codes: 0 held, 1 a file differs, 2 the manifest is missing or unreadable.
- Example: `cargo xtask verify reference-untouched`

## Boundaries

- Depends on: `cargo_metadata`, `sha2`, `walkdir`, `ignore`, the layout tables in the crate's
  layout module.
- Used by: the orchestrator, every agent's stage gate, `cargo xtask ci ci-local`.
- Rules:
  - every gate has a fixture test that shows it holding and failing (`tests/`);
  - the gate names are stable: CLAUDE.md, the standards and the briefs cite them.

## Related documentation

- [Laws and gates](/documentation/program/laws_and_gates.md) — what each gate holds and why.
````
