# Reference mods

The two original Task Force Arrowhead Radio mods, kept byte for byte as the specification the Rust port
follows. Nothing here is built, formatted or edited; the port reads these files and the behaviour specs
in `documentation/program/behaviour_specs/` record what they do.

## Contents

```text
reference/
├── MANIFEST.sha256  sha256 of every file in both trees, written by cargo xtask reference-manifest
├── tfar_0_9_12/     the TFAR 0.9.12 release: SQF addons, the C++ plugin, the C++ pipe extension;
│                    local only, gitignored here
└── tfar_1_0/        upstream master at fce977fa, a git clone with its own .git, gitignored here
```

## How it works

- **tfar_1_0** is a clean clone of `https://github.com/michail-nikolaev/task-force-arma-3-radio` at
  `origin/master`, commit `fce977fa` ("Add hemtt support"). It keeps its own `.git` as its backup. The
  outer repository gitignores the whole tree (decision D15).
- **tfar_0_9_12** is the 0.9.12 release (about 22 MB, 919 files), which has no version control of its
  own. The outer repository gitignores it too (decision D15): its images and models are upstream Git LFS
  pointer stubs whose objects exist nowhere, and a host such as GitHub refuses a push that references
  them. The owner keeps the tree locally; the manifest guards it.

Both trees are **read-only**: never write, format, move, rename or delete a file under `reference/`. This
README is the one file the port writes here (law 4 in [CLAUDE.md](/CLAUDE.md#1-core-project-laws)).

The manifest, `MANIFEST.sha256`, holds one sorted `sha256  relative/path` line for every file under
`reference/` except this README, the manifest itself and everything under `tfar_1_0/.git/`.
`cargo xtask reference-manifest` writes it and refuses to overwrite it without `--force` and a reason;
`cargo xtask verify reference-untouched` hashes the trees again and reports every added, removed and
changed file. Because the 1.0 tree is not in the outer repository, the manifest is its only guard.

### Restore tfar_1_0

A fresh clone of the outer repository has no `reference/tfar_1_0/`. Restore it from upstream, from the
repository root:

```bash
git clone https://github.com/michail-nikolaev/task-force-arma-3-radio reference/tfar_1_0
git -C reference/tfar_1_0 checkout fce977fa
cargo xtask verify reference-untouched
```

The last command holds when the restored tree matches the manifest.

### Restore tfar_0_9_12

A fresh clone of the outer repository has no `reference/tfar_0_9_12/` either. Copy the 0.9.12 release
tree into place from the owner's local copy, then run `cargo xtask verify reference-untouched`; it holds
when every file matches the manifest. Until both trees are in place the gate reports the missing files.

`tfar_0_9_12` holds upstream Git LFS pointer stubs: 130-byte text files in place of `.paa`, `.png`, `.p3d`
and `.psd` files, plus nested `.gitattributes` files that mark those types `filter=lfs`. The stubs stay as
plain bytes, so git-lfs must never run on them; the working clone keeps this line in
`.git/info/attributes` as a guard:

```text
reference/** -filter -diff -merge -text
```

## Boundaries

- Depends on: upstream TFAR (`https://github.com/michail-nikolaev/task-force-arma-3-radio`) for the 1.0
  tree; nothing for the 0.9.12 tree.
- Used by: the behaviour specs and the crate catalogue, which cite these files by path and line; the
  golden-vector tool, which compiles DSPFilters, SimpleComp and clunk straight from here; `cargo xtask
  dist`, which copies `radio-sounds/` from each tree's `ts/` folder into target/dist/ without changing
  them.
- Rules: every file stays byte-identical to the manifest (`cargo xtask verify reference-untouched`); no
  gate, formatter or editor setting touches this folder (the root .editorconfig and typos.toml exclude
  it).

## Related documentation

- [Workspace decisions](/documentation/decisions/workspace.md) — why the mods live here (D5) and how the
  1.0 clone is guarded (D15).
- [Behaviour specs](/documentation/program/behaviour_specs/README.md) — what the original plugins and
  extensions do, cited by file and line.
