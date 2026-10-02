# Applications

The four deliverables of the TFAR Rust port: for each mod version, one TeamSpeak 3 plugin (a native
Linux shared library) and one Arma 3 extension (a Windows x64 DLL run under Proton). Each is a
cdylib, plus an rlib so host tests and the mock TeamSpeak host link the same code.

## Contents

```text
apps/
├── tfar_extension_0_9/  the 0.9.12 `task_force_radio_pipe` extension DLL, crate `tfar_extension_0_9`
├── tfar_extension_1_0/  the 1.0 `task_force_radio_pipe` extension DLL, crate `tfar_extension_1_0`
├── tfar_plugin_0_9/     the 0.9.12 TeamSpeak plugin, crate `tfar_plugin_0_9`
└── tfar_plugin_1_0/     the 1.0 TeamSpeak plugin, crate `tfar_plugin_1_0`
```

## How it works

Arma calls its extension through `callExtension`; the extension forwards each call over localhost
TCP to the plugin of the same version, which TeamSpeak has loaded. The plugin answers sync calls,
queues async ones, and shapes every voice frame from the state the game sends. The two versions
listen on different ports, so both plugins can be loaded at once; a plugin whose link never
connected stays dormant.

```text
Arma 3 (Proton) -> tfar_extension_X -- TCP 127.0.0.1 --> tfar_plugin_X <- TeamSpeak 3 (Flatpak)
                   Windows DLL         0.9: 47810         Linux .so
                                       1.0: 47820
```

The cdylib files keep their crate names in `target/`; `cargo xtask dist` renames them to the names
the mods and TeamSpeak expect. Nothing copies them into a mod or TeamSpeak folder.

## Getting started

Run from the repository root:

```bash
cargo build -p tfar_plugin_0_9 -p tfar_plugin_1_0                                     # .so files in target/debug/
cargo build -p tfar_extension_0_9 -p tfar_extension_1_0 --target x86_64-pc-windows-gnu  # DLLs in target/x86_64-pc-windows-gnu/debug/
cargo test -p tfar_plugin_0_9 -p tfar_plugin_1_0 -p tfar_extension_0_9 -p tfar_extension_1_0
```

## Boundaries

- Depends on: the library crates in `crates/`; TeamSpeak 3.6 (plugin API 26) and Arma 3 at run
  time.
- Used by: TeamSpeak 3 and Arma 3, which load the built files; `tools/tfar_ts3_mock` and
  `tools/tfar_extension_probe` in tests.
- Rules: a plugin app never depends on the other version's crates; an extension app depends only on
  foundation and game_link crates; no crate depends on an app (`cargo xtask verify crate-tiers`);
  `unsafe` is allowed here for export glue only (`cargo xtask verify unsafe-allowlist`).

## Related documentation

- [Arma extensions](/documentation/program/behaviour_specs/extensions.md) — the original
  extensions and the Arma ABI facts.
- [Windows dependencies](/documentation/program/behaviour_specs/windows_dependencies.md) — why the
  originals cannot connect under Proton and what replaces their Win32 objects.
