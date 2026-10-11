# Arma extension probe

The `tfar_extension_probe` crate: a Windows executable that loads an extension DLL and calls
`RVExtension`, run under Proton's wine. Developers run it under Proton's wine against a built
extension.

## How it works

The stand-in for Arma. It loads the extension library with `libloading` (`LoadLibrary` on Windows,
`dlopen` on Linux), resolves `RVExtension` and the optional `RVExtensionVersion` by their
undecorated names, prints the version, then makes each call with a zeroed 10240-byte output
buffer, the size Arma passes, and prints the reply up to its NUL. Built for
`x86_64-pc-windows-gnu` it runs under Proton's wine against the DLL; built for Linux it loads the
`.so` build of an extension, so the extension's own logic can be probed on the host:

```text
$ tfar_extension_probe target/debug/libtfar_extension_1_0.so 'DFRAME~'
library: target/debug/libtfar_extension_1_0.so
version: tfar_extension_1_0 0.1.1
call:  DFRAME~
reply: Not connected to TeamSpeak
```

`main.rs` parses the arguments with `probe_arguments::parse_arguments` (usage error: exit 2), loads
the library with `ExtensionLibrary::load`, prints `version()` and then each `call(text)`; any error
on the way ends the run with exit 1. `extension_library.rs` holds every `unsafe` block: the
library load, the two symbol lookups and the two foreign calls, each with its `// SAFETY:`
comment. `reply_from_buffer` reads the reply up to the first NUL and is tested on its own. Each
module attaches its sibling test file under `tests/` with `#[path]`.

## Getting started

Run from the repository root:

```bash
cargo build -p tfar_extension_probe --target x86_64-pc-windows-gnu  # tfar_extension_probe.exe under target/x86_64-pc-windows-gnu/debug/
cargo test -p tfar_extension_probe                                  # the host unit tests
cargo build -p tfar_extension_1_0 && cargo run -p tfar_extension_probe -- target/debug/libtfar_extension_1_0.so
```

## Configuration

None: the probe reads only its command line.

## Public surface

- The binary: `tfar_extension_probe <path-to-extension-library> [call ...]`. With no call it sends
  `TS_INFO\tPING` and `TS_INFO\tVERSION`. In a call, the two characters `\t` stand for a TAB (the
  field separator of every TFAR call); nothing else is changed. Replies are printed with control
  characters escaped. Exit codes: 0 when every call was made, 1 when the library did not load, has
  no `RVExtension` or a call holds a NUL byte, 2 for a command line without a library path (or
  `-h`, `--help`).

## Boundaries

- Depends on: no workspace crate; `libloading` (loading the library and resolving its exports) and
  `anyhow` (errors with context).
- Used by: developers, by hand, under Proton's wine or on the Linux host.
- Rules: the call text reaches the extension unchanged but for the `\t` escape, and the reply is
  printed as the extension wrote it (`src/tests/`); the probe never depends on the extension apps
  (tools never use the deliverables). `unsafe` is
  allowed here for loading and calling the library (`cargo xtask verify unsafe-allowlist`).

## Related documentation

- [Arma extensions](/documentation/behaviour_specs/extensions.md) — the original
  `task_force_radio_pipe` extensions and the Arma ABI facts.
- [Game link protocol](/documentation/architecture/game_link_protocol.md) — the replies an
  extension gives with and without a plugin.
