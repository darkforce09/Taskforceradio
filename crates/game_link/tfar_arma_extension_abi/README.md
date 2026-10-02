# Arma extension ABI

The `tfar_arma_extension_abi` crate: the `RVExtension` exports and the Win32 calls an Arma extension
DLL needs. Used by `tfar_extension_0_9`, `tfar_extension_1_0`.

## Contents

```text
crates/game_link/tfar_arma_extension_abi/
├── Cargo.toml  the `tfar_arma_extension_abi` package (lib): tier 0, `windows-sys` on Windows only
└── src/        the export macro, the reply writer, the command-line and foreground-window queries
```

## How it works

An extension app expands `arma_extension!(handler: answer_game_call, version: "1.0.0")` once at its
crate root. The expansion writes the two undecorated exports Arma x64 resolves:
`RVExtension(output, output_size, function)` and `RVExtensionVersion(output, output_size)`. Each
export is one call into `answer_extension_call` or `answer_extension_version`, which check for null
pointers and non-positive sizes, read the input as bytes up to its NUL, decode it lossily (valid
UTF-8 passes unchanged), call the handler and write the reply with `write_reply`. `write_reply`
keeps at most `len − 1` bytes, cuts back to a UTF-8 character boundary and writes the NUL.

On x86_64 Windows `extern "system"` is the C ABI, so the export names stay undecorated. No 32-bit
build is supported.

`arma_command_line_contains` reads `GetCommandLineW` (the `-tfdebug` test of the originals), and
`foreground_window_is_arma` reads `GetForegroundWindow` and `GetClassNameW` and looks for `arma`,
ignoring case (the test at `reference/tfar_1_0/ts/src/Teamspeak.cpp:207`). Off Windows the first
reads `std::env::args` and the second answers `false`, so the crate builds and tests on Linux.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_arma_extension_abi                                # the unit tests
cargo build -p tfar_arma_extension_abi --target x86_64-pc-windows-gnu  # the Windows build
```

## Configuration

None: the crate reads no setting.

## Public surface

- `arma_extension!`: writes `RVExtension` and `RVExtensionVersion` in the expanding crate.
- `answer_extension_call`, `answer_extension_version`: the checked bodies of the two exports.
- `write_reply`: writes a reply truncated to a buffer, always NUL-terminated when there is room.
- `arma_command_line_contains`: whether the process command line contains a flag.
- `foreground_window_is_arma`: whether the window in front is Arma's.
- `CRATE_ROLE`, `Error`, `Result` and `prelude`.

## Boundaries

- Depends on: no workspace crate; `thiserror` for the error type; `windows-sys` on Windows
  (`Win32_System_Environment`, `Win32_UI_WindowsAndMessaging`).
- Used by: `tfar_extension_0_9`, `tfar_extension_1_0`.
- Rules: one of the unsafe-allow-listed crates; every `unsafe` block carries a `// SAFETY:`
  comment. A reply never writes past the buffer Arma passes. Tier 0 in `crates/game_link`: every
  workspace dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [Arma extensions](/documentation/program/behaviour_specs/extensions.md) — the original
  `task_force_radio_pipe` extensions and the Arma ABI facts.
- [Game link protocol](/documentation/architecture/game_link_protocol.md) — the transport the
  extensions speak and the reply truncation rule.
- [Windows dependencies](/documentation/program/behaviour_specs/windows_dependencies.md) — the Win32
  objects the originals use and what replaces them.
