# Arma extension ABI source

The source of `tfar_arma_extension_abi`: the `RVExtension` export macro and its checked bodies, the
reply writer and the two Win32 queries.

## Contents

```text
crates/game_link/tfar_arma_extension_abi/src/
├── command_line.rs       `arma_command_line_contains`: `GetCommandLineW`, or `std::env::args` off Windows
├── crate_role.rs         `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs              `Error`, the `thiserror` enum, and `Result`
├── extension_exports.rs  `arma_extension!`, `answer_extension_call`, `answer_extension_version`
├── foreground_window.rs  `foreground_window_is_arma`: the class-name test behind `FOCUS`
├── lib.rs                the crate root: module header, `#![allow(unsafe_code)]`, `mod` lines, re-exports
├── prelude.rs            the macro, the reply writer and the two queries for glob import
├── reply_buffer.rs       `write_reply`: truncation at a character boundary, then the NUL
└── tests/                unit tests, one file per module
```

## How it works

`extension_exports.rs` holds every pointer check, so the exports the macro writes are one `unsafe`
call each; `reply_buffer.rs` is the safe core both exports write through. The Win32 calls sit behind
`#[cfg(windows)]` in `command_line.rs` and `foreground_window.rs`, each with a portable fallback.
`tests/extension_exports.rs` expands the macro and calls the exports through raw function pointers
with Rust-allocated buffers, as Arma calls them.

## Public surface

- `arma_extension!`, `answer_extension_call`, `answer_extension_version`, `write_reply`,
  `arma_command_line_contains`, `foreground_window_is_arma`, `CRATE_ROLE`, `Error`, `Result` and
  `prelude`, re-exported at the crate root.

## Boundaries

- Depends on: `thiserror`; `windows-sys` on Windows.
- Used by: `tfar_extension_0_9`, `tfar_extension_1_0`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); every
  `unsafe` block has a `// SAFETY:` comment; tests live in `tests/`
  (`cargo xtask verify test-placement`).
