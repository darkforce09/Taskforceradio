# Game link crates

The localhost TCP link between the Arma extensions (Windows DLLs under Proton) and the native
TeamSpeak plugins, and the Arma side of an extension DLL.

## How it works

`tfar_game_link_protocol` defines the frames. The plugins run `tfar_game_link_server`, the
extensions run `tfar_game_link_client`, and `tfar_arma_extension_abi` holds the `RVExtension`
exports and the Win32 focus query the extensions need.

```text
Arma (Proton) -- callExtension --> extension DLL -- TCP 127.0.0.1 --> TeamSpeak plugin (.so)
                                  game_link_client                    game_link_server
```

## Getting started

Run from the repository root:

```bash
# the unit tests of every crate in this category
cargo test \
  -p tfar_arma_extension_abi \
  -p tfar_game_link_client \
  -p tfar_game_link_protocol \
  -p tfar_game_link_server
```

## Boundaries

- Depends on: `crates/foundation` and the game_link category itself.
- Used by: all four apps and `tools/tfar_ts3_mock`.
- Rules: `postcard` stays in game_link crates and `windows-sys` in `tfar_arma_extension_abi`, which
  is on the unsafe allow-list (`cargo xtask verify crate-tiers` holds the edges,
  `cargo xtask verify dependency-firewalls` the external crates); each crate declares
  `category = "crates/game_link"` and its tier: 0 without a workspace dependency, else one above the
  highest.

## Related documentation

- [Arma extensions](/documentation/behaviour_specs/extensions.md) — the original extensions and the Arma ABI facts.
- [Windows dependencies](/documentation/behaviour_specs/windows_dependencies.md) — the Win32 objects the link replaces.
