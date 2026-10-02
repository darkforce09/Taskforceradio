**Status:** live

# README template: leaf

**When to use:** a folder with no child folders besides `tests/`, such as a Rust module folder that holds
only source files. The [README standard](/documentation/standards/readme_standard.md) defines every rule
this template follows; the leaf kind adds no sections of its own.

## Skeleton

Copy the block and replace every `<…>` placeholder; each one says what goes there. A leaf of at most
three files, README.md not counted, may leave out How it works.

````markdown
# <What the folder holds, in plain words: no path, no backticks>

<One to three sentences: what this folder is for.>

## Contents

```text
<repository path of the folder>/
├── <file name or glob>  <what it is for: a lowercase phrase, no closing period>
└── tests/               <what the unit tests cover>
```

## How it works

<How the files work together: the flow through them, the main types, the invariants that span files.
Leave the section out when the folder holds at most three files.>

## Boundaries

- Depends on: <the modules, crates and files this folder uses, read from its imports>
- Used by: <every user outside the folder, found with git grep; "nothing" when none>
- Rules: <the invariants a change here must keep, and the test or gate that checks each>

## Related documentation

- [<document title>](/documentation/<path to the document>) — <what it covers; leave the section out
  when no document goes deeper>
````

## Worked sample

Written for the frame kinds module of `tfar_game_link_protocol`, as it reads once its code is written.

````markdown
# Frame kinds

The kinds of frame the extension and the plugin exchange on the
[game link](/documentation/glossary.md#game-link), one file per direction.

## Contents

```text
crates/game_link/tfar_game_link_protocol/src/frame_kinds/
├── extension_to_plugin.rs  HELLO, SYNC_REQ, ASYNC and FOCUS, with their payloads
├── mod.rs                  the module tree and the `FrameKind` byte values
├── plugin_to_extension.rs  SYNC_RESP and STATUS, with their payloads
└── tests/                  round trips of every kind and the rejection of unknown bytes
```

## How it works

`FrameKind` holds the one byte after the length prefix. Each direction's file defines the payload types
for its kinds and their `postcard` encoding; `mod.rs` maps bytes to kinds and refuses any byte it does
not know.

## Boundaries

- Depends on: `serde`, `postcard`, the crate's `error.rs`.
- Used by: `frame.rs` in the same crate; `tfar_game_link_server` and `tfar_game_link_client`.
- Rules:
  - every kind round-trips byte for byte (`every_frame_kind_round_trips`);
  - an unknown kind byte is an error value (`an_unknown_kind_byte_is_rejected`).

## Related documentation

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — frame layout and semantics.
````
