# Arma extension probe source

The source of `tfar_extension_probe`: its `main` and its role constant. The probe's own code is not
written yet.

## Contents

```text
tools/tfar_extension_probe/src/
├── crate_role.rs  `CRATE_ROLE`: what the probe is for, in one sentence
├── main.rs        the module header and `main`, which prints the role
└── tests/         unit tests of `CRATE_ROLE`
```

## How it works

`main.rs` declares `crate_role` and prints `CRATE_ROLE`; `crate_role.rs` attaches its sibling test
file `tests/crate_role.rs` with `#[path]`.

## Public surface

- The binary's `main`: prints `CRATE_ROLE`; developers run it.

## Boundaries

- Depends on: the standard library only.
- Used by: nothing; the binary is its own entry point.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` (`cargo xtask verify test-placement`).
