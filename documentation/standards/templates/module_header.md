**Status:** live

# Template: module header

**When to use:** the `//!` header that opens every crate root (`lib.rs`, `main.rs`) and every production
`.rs` file of 80 lines or more (`cargo xtask verify module-headers`). Shorter files may carry one too. The
header is the module's contract: what it does, where it sits, what state and threads it touches, and
what always holds. The [coding standards](/documentation/standards/coding_standards.md#comments) set the
rules around it.

## Skeleton

Copy the block and replace every `<…>` placeholder; each one says what goes there. The four labels are
spelled exactly as here, in this order, each opening its own paragraph. An empty point says so ("none;
pure functions").

```rust
//! <One line: what the module is, in plain words.>
//!
//! **Role:** <the module's one responsibility in its crate>
//!
//! **Position:** <where it sits: what feeds it, who calls it, what it calls, in which category and tier>
//!
//! **Signals and state:** <the mutable state it owns or touches, and the threads that call it: the
//! TeamSpeak audio thread, the game link thread, the service tick; or "none; pure functions">
//!
//! **Invariants:** <what always holds: units, ranges, ordering, ownership, the original behaviour it
//! keeps, each worded so a test can check it>
```

Write in the present tense. The header says what the module does now; it never says what it replaced,
when it was written, or which milestone or finding it serves. A ported behaviour may name the
`reference/` file it follows as its specification.

## Worked sample

The header of the frame codec of `tfar_game_link_protocol`, as it reads once its code is written.

```rust
//! The length-prefixed frame of the game link.
//!
//! **Role:** encodes a frame kind and its payload into bytes, and decodes bytes back into a frame,
//! refusing anything malformed.
//!
//! **Position:** tier 0 of crates/game_link; called by `tfar_game_link_server` in the plugins and
//! `tfar_game_link_client` in the extensions, on either side of the localhost TCP socket.
//!
//! **Signals and state:** none; pure functions over byte slices, called from whichever thread owns
//! the socket.
//!
//! **Invariants:** a frame is `u32 LE length | u8 kind | payload`, and the length counts the kind byte
//! and the payload; a length above `MAX_FRAME_LENGTH` is rejected before any allocation; every frame
//! that encodes decodes to itself.
```
