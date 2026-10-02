# Game link protocol source

The source of `tfar_game_link_protocol`: the frame kinds, their byte encoding, the streaming decoder
and the blocking helpers both ends of the game link use.

## Contents

```text
crates/game_link/tfar_game_link_protocol/src/
├── crate_role.rs          `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs               `Error`: zero length, over length, unknown kind, bad payload, input or output
├── flavour.rs             `Flavour`, the mod version byte of `HELLO`
├── frame.rs               `Frame`, `FrameKind` and the `HELLO` values with their mismatch check
├── frame_codec.rs         `encode`, `frame_body_length` and `decode_frame_body`
├── frame_decoder.rs       `FrameDecoder`: arbitrary byte chunks in, complete frames out
├── frame_io.rs            `read_frame` and `write_frame` over `std::io` streams
├── game_link_port.rs      `port(flavour, debug)` and the four port constants
├── lib.rs                 the crate root: module header, `mod` lines and re-exports
├── prelude.rs             the frame, codec and flavour names for glob import
├── protocol_constants.rs  `MAGIC`, `PROTOCOL_VERSION`, `MAX_FRAME_LENGTH`, the prefix size
└── tests/                 unit tests: round trips, golden bytes, byte-at-a-time decoding, each error
```

## How it works

`frame_codec.rs` writes a placeholder length, the kind byte from `frame.rs` and the `postcard`
payload, then fills in the length, refusing a body above `MAX_FRAME_LENGTH`. Decoding checks the
length first (`frame_body_length`), then maps the kind byte and decodes the payload, refusing bytes
left over. `frame_decoder.rs` buffers chunks and runs the same two steps once a whole frame is in;
`frame_io.rs` reads one frame with `read_exact`, allocating the body only after its length passed.
Text fields are byte strings (`postcard` writes a varint length, then the bytes).

## Public surface

- `Frame`, `FrameKind`, `Hello`, `HelloMismatch`, `Flavour`: the values of the wire table.
- `encode`, `decode_frame_body`, `frame_body_length`, `FrameDecoder`, `read_frame`, `write_frame`.
- `port`, `PORT_0_9`, `PORT_0_9_DEBUG`, `PORT_1_0`, `PORT_1_0_DEBUG`, `MAGIC`, `PROTOCOL_VERSION`,
  `MAX_FRAME_LENGTH`, `FRAME_LENGTH_PREFIX_SIZE`, `Error`, `Result`, `CRATE_ROLE`, `prelude`.

## Boundaries

- Depends on: `postcard`, `serde`, `thiserror`.
- Used by: `tfar_game_link_client`, `tfar_game_link_server`, `tfar_ts3_mock`.
- Rules: the kind codes and payload field order are the protocol table's, held by the golden byte
  tests in `tests/frame_codec.rs`; production files hold no `unwrap`, `expect`, `panic` or indexing
  (clippy deny list); tests live in `tests/` (`cargo xtask verify test-placement`).
