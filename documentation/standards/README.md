**Status:** live

# Standards

The rules for code, documents, READMEs and commits, and the templates that put them into practice. They
adapt the TBD-Reforger and TBD-subtitles standards to a workspace of two TeamSpeak plugins, two Arma
extensions, their shared libraries and their tools.

## Contents

```text
documentation/standards/
├── coding_standards.md          Rust rules: manifests, dependencies, layering, panics, FFI, threads, tests
├── commit_conventions.md        commit messages and scopes, what goes in one commit, who commits
├── documentation_standards.md   where documents live, status lines, program records, writing, lifecycle
├── readme_standard.md           the README core, the Contents grammar, the folder kinds, writing rules
└── templates/                   one skeleton per README kind and document type, with worked samples
```

## How it works

[CLAUDE.md](/CLAUDE.md) states the laws in one paragraph each; these documents give the detail, and the
templates turn them into skeletons to copy. Every rule a program can check names the
`cargo xtask verify` gate, the lint or the test that holds it; the rest are held in review by the
orchestrator when it reads a slice. The gates themselves, with their exit codes and the perturbation
proofs that show they bite, are listed in [laws and gates](/documentation/program/laws_and_gates.md).

## Code

- The gates behind these standards live in tools/xtask, in its verify command group.
- The workspace lints and profiles they rely on live in the root Cargo manifest.

## Boundaries

- Depends on: the laws in [CLAUDE.md](/CLAUDE.md) and the [decision log](/documentation/decisions/README.md),
  above these standards on the authority ladder.
- Used by: every README, document, source file and commit in the repository; `cargo xtask new-crate`,
  which writes a crate to the anatomy the coding standards fix; every agent brief.
- Rules: a rule marked as gated names its gate, and the gate exists in `cargo xtask verify`; no document
  here passes 500 lines (`cargo xtask verify markdown-placement`).

## Related documentation

- [Documentation map](/documentation/README.md) — where each kind of document lives and the authority
  ladder.
- [Laws and gates](/documentation/program/laws_and_gates.md) — every gate, its exit codes and its proof.
