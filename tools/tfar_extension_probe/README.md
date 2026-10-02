# Arma extension probe

The `tfar_extension_probe` crate: a Windows executable that loads an extension DLL and calls
`RVExtension`, run under Proton's wine. Developers run it under Proton's wine against a built
extension.

## Contents

```text
tools/tfar_extension_probe/
├── Cargo.toml  the `tfar_extension_probe` package (bin): tier 0, workspace lints and dependencies
└── src/        the probe's `main`, its role constant and tests
```

## How it works

The stand-in for Arma: a Windows executable that loads an extension DLL with `LoadLibrary`, calls
`RVExtension` with a message and prints the reply, run under Proton's wine. The code is not written
yet: `src/main.rs` carries the module header and prints the role that `src/crate_role.rs` states.

## Getting started

Run from the repository root:

```bash
cargo build -p tfar_extension_probe --target x86_64-pc-windows-gnu  # tfar_extension_probe.exe under target/x86_64-pc-windows-gnu/debug/
cargo test -p tfar_extension_probe                                  # the host unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the binary prints its role; the surface it is for:

- the binary tfar_extension_probe.exe: `tfar_extension_probe <dll> <message>` prints the
  extension's reply.

## Boundaries

- Depends on: no workspace crate.
- Used by: developers, by hand, under Proton's wine.
- Rules: the probe passes the message unchanged and prints the reply unchanged. Tier 0 in `tools`:
  every workspace dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [Arma extensions](/documentation/program/behaviour_specs/extensions.md) — the original
  `task_force_radio_pipe` extensions and the Arma ABI facts.
