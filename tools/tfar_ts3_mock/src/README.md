# Mock TeamSpeak host source

The source of `tfar_ts3_mock`: the crate's role constant, its error type and its prelude. The
crate's own code is not written yet.

## Contents

```text
tools/tfar_ts3_mock/src/
├── crate_role.rs  `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs       `Error`, the `thiserror` enum, and `Result`
├── lib.rs         the crate root: module header, `mod` lines and re-exports
├── main.rs        the binary: prints the library's `CRATE_ROLE`
├── prelude.rs     `CRATE_ROLE` for glob import
└── tests/         unit tests of `CRATE_ROLE`
```

## How it works

`lib.rs` declares the private modules `crate_role` and `error` and re-exports `CRATE_ROLE`, `Error`
and `Result` at the crate root; `prelude.rs` re-exports `CRATE_ROLE` only, so a glob import never
shadows `std`'s `Result`. `crate_role.rs` attaches its sibling test file `tests/crate_role.rs` with
`#[path]`. `main.rs` links the library and prints its role.

## Public surface

- `CRATE_ROLE`, `Error`, `Result` and `prelude`, re-exported at the crate root.
- The binary's `main`: prints `CRATE_ROLE`; developers run it.

## Boundaries

- Depends on: `thiserror`.
- Used by: the plugin apps' test suites.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` (`cargo xtask verify test-placement`).
