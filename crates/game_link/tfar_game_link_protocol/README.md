# Game link protocol

The `tfar_game_link_protocol` crate: the frames the Arma extensions and the TeamSpeak plugins
exchange over localhost TCP. Used by `tfar_game_link_client`, `tfar_game_link_server`,
`tfar_ts3_mock`.

## Contents

```text
crates/game_link/tfar_game_link_protocol/
├── Cargo.toml  the `tfar_game_link_protocol` package (lib): tier 0, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The wire format of the game link: `u32` little-endian length, a kind byte and a `postcard` payload,
the `HELLO` handshake, the frame kinds and the ports each plugin listens on. The code is not written
yet: `src/lib.rs` carries the module header and re-exports, `src/crate_role.rs` states the crate's
role in one constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_game_link_protocol  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `GameLinkFrame`: `HELLO`, `SYNC_REQ`, `SYNC_RESP`, `ASYNC`, `FOCUS` and `STATUS`.
- `encode_frame` and `decode_frame` with the length prefix.
- `GameLinkPort`: 47810/47811 for 0.9 and 47820/47821 for 1.0.

## Boundaries

- Depends on: no workspace crate; `thiserror` for the error type. Planned externals: `postcard`,
  `serde`.
- Used by: `tfar_game_link_client`, `tfar_game_link_server`, `tfar_ts3_mock`.
- Rules: a frame decodes back to the frame that was encoded, and an oversized length is refused.
  Tier 0 in `crates/game_link`: every workspace dependency has a lower tier and follows the edge
  matrix in `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [Arma extensions](/documentation/program/behaviour_specs/extensions.md) — the original
  `task_force_radio_pipe` extensions and the Arma ABI facts.
- [Windows dependencies](/documentation/program/behaviour_specs/windows_dependencies.md) — the Win32
  objects the originals use and what replaces them.
