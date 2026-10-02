# Arma extension probe source

The source of `tfar_extension_probe`: its `main`, the command-line reader and the extension library
loader that calls the exports the way Arma does.

## Contents

```text
tools/tfar_extension_probe/src/
├── crate_role.rs         `CRATE_ROLE`: what the probe is for, in one sentence
├── extension_library.rs  `ExtensionLibrary`: load, resolve `RVExtension`/`RVExtensionVersion`, call
├── main.rs               the module header and `main`: parse, load, print the version and each reply
├── probe_arguments.rs    the command line: the library path, the calls, the `\t` escape, the usage
└── tests/                unit tests of the command line, the reply buffer and `CRATE_ROLE`
```

## How it works

`main.rs` parses the arguments with `probe_arguments::parse_arguments` (usage error: exit 2), loads
the library with `ExtensionLibrary::load`, prints `version()` and then each `call(text)`; any error
on the way ends the run with exit 1. `extension_library.rs` holds every `unsafe` block: the
library load, the two symbol lookups and the two foreign calls, each with its `// SAFETY:`
comment. `reply_from_buffer` reads the reply up to the first NUL and is tested on its own. Each
module attaches its sibling test file under `tests/` with `#[path]`.

## Public surface

- The binary's `main`; developers run it. Nothing else crosses the crate boundary.

## Boundaries

- Depends on: `libloading`, `anyhow`.
- Used by: nothing; the binary is its own entry point.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` (`cargo xtask verify test-placement`); `unsafe` stays in
  `extension_library.rs`.
