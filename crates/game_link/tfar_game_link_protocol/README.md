# Game link protocol

The `tfar_game_link_protocol` crate: the frames the Arma extensions and the TeamSpeak plugins
exchange over localhost TCP. Used by `tfar_game_link_client`, `tfar_game_link_server`,
`tfar_ts3_mock`.

## How it works

A frame on the wire is a `u32` little-endian length, a kind byte and the `postcard` encoding of the
kind's fields in table order. `src/` holds the in-memory `Frame`, the encoder, a decoder for whole
bodies, a streaming `FrameDecoder` for sockets that deliver bytes in arbitrary chunks, and blocking
`read_frame` and `write_frame` helpers. Every malformed input (a zero or oversized length, an unknown
kind, a payload that does not decode exactly) is an `Error`, after which the receiver closes the
connection.

`frame_codec.rs` writes a placeholder length, the kind byte from `frame.rs` and the `postcard`
payload, then fills in the length, refusing a body above `MAX_FRAME_LENGTH`. Decoding checks the
length first (`frame_body_length`), then maps the kind byte and decodes the payload, refusing bytes
left over. `frame_decoder.rs` buffers chunks and runs the same two steps once a whole frame is in;
`frame_io.rs` reads one frame with `read_exact`, allocating the body only after its length passed.
Text fields are byte strings (`postcard` writes a varint length, then the bytes).

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_game_link_protocol  # round trips, golden bytes, the decoder and every error class
```

## Configuration

None: the crate reads no setting.

## Public surface

- `Frame` (`Hello`, `SyncRequest`, `SyncResponse`, `Async`, `Focus`, `Status`), `FrameKind`,
  `Hello` and `HelloMismatch`: the frames and the handshake check.
- `encode`, `decode_frame_body`, `frame_body_length`, `FrameDecoder`, `read_frame`, `write_frame`:
  the codec.
- `Flavour` (`V0_9 = 9`, `V1_0 = 10`), `port(flavour, debug)` and the four port constants: 47810 and
  47811 for 0.9, 47820 and 47821 for 1.0.
- `MAGIC`, `PROTOCOL_VERSION`, `MAX_FRAME_LENGTH`, `FRAME_LENGTH_PREFIX_SIZE`, `Error`, `Result`,
  `prelude`.

## Boundaries

- Depends on: no workspace crate; `postcard` (`use-std`) for the payloads, `serde` (`derive`) for
  the `HELLO` payload, `thiserror` for the error type.
- Used by: `tfar_game_link_client`, `tfar_game_link_server`, `tfar_ts3_mock`.
- Rules: every frame that encodes decodes to itself, and the `HELLO` and
  `SYNC_REQ{1, "TS_INFO\tPING"}` bytes match their golden vectors (`src/tests/frame_codec.rs`); a
  length of zero or above 1 MiB is refused before the body is read. Tier 0 in `crates/game_link`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — the authoritative frame
  table, handshake, timeouts and ports.
- [Arma extensions](/documentation/behaviour_specs/extensions.md) — the original
  `task_force_radio_pipe` extensions and the Arma ABI facts.
