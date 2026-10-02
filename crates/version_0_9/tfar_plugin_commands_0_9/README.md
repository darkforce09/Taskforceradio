# Plugin commands for 0.9

The `tfar_plugin_commands_0_9` crate: the 0.9.12 plugin-to-plugin commands sent through TeamSpeak,
the sender filter and the client metadata block, byte-compatible with the original. Used by
`tfar_plugin_0_9`.

## Contents

```text
crates/version_0_9/tfar_plugin_commands_0_9/
├── Cargo.toml  the `tfar_plugin_commands_0_9` package (lib): tier 2, workspace lints and dependencies
└── src/        the plugin commands, the sender filter, the metadata block and the error type
```

## How it works

Players' plugins talk through TeamSpeak's `sendPluginCommand` to the current channel, and show
their state in the `<TFAR>…</TFAR>` block of their client metadata. Both texts must stay byte for
byte those of `reference/tfar_0_9_12/ts/src/plugin.cpp`, so a Linux player and Windows players
with the original plugin read each other.

| Command | Text | Original |
|---|---|---|
| tangent | `TANGENT\|TANGENT_LR\|TANGENT_DD`, `PRESSED\|RELEASED`, frequency, range, subtype, nickname | plugin.cpp:1837, read at 3076 |
| release all | `RELEASE_ALL_TANGENTS`, nickname | plugin.cpp:1787, read at 3080 |
| volume | `VOLUME`, nickname, voice volume, `true\|false` | plugin.cpp:2906, read at 3084 |

Fields are tab-separated and split like `std::getline`. On decode a tangent's range is rounded and
the voice volume truncated as the original reads them; `PRESSED` and `true` are compared exactly.
The encoder writes the range as an integer, which every 0.9.12 decoder reads to the same value.
A command is taken only from a plugin whose name starts with `task_force_radio`
(`accepts_sender`, the `strncmp` of plugin.cpp:3116).

The status text is
`[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]Y[/B] [I]P:[/I][B]0.9.12[/B][I] A: [/I][B]<addon>[/B]`
(plugin.cpp:1300-1304), with `N` for a false flag; at shutdown it is
`[B]Task Force Radio Plugin Disabled[/B]`. Another client counts as playing when its block is
longer than `[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]Y` and starts with it (plugin.cpp:2531).
`merge_metadata` replaces the text of an existing block and keeps what surrounds it; without a
block the result is the block alone, as the original's `setMetaData` writes it.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_plugin_commands_0_9  # byte-exact texts, round trips, the sender filter, metadata
```

## Configuration

None: the crate reads no setting.

## Public surface

- `PluginCommand` with `encode` and `decode`, `TangentKind`, and the command names in
  `plugin_command`.
- `PLUGIN_NAME_PREFIX`, `accepts_sender`.
- `format_metadata`, `merge_metadata`, `metadata_block`, `metadata_says_playing`,
  `info_data_text`, `DISABLED_METADATA`, and the tags and sizes in `metadata`.
- `Error`, `Result`, `CRATE_ROLE`, `prelude`.

## Boundaries

- Depends on: `tfar_arma_values` (field splitting, numbers, `RadioSubtype`); `thiserror` for the
  error type.
- Used by: `tfar_plugin_0_9`.
- Rules: the encoded texts match what the original 0.9.12 plugin sends, held by the byte-exact
  tests in `src/tests/plugin_command.rs` and `src/tests/metadata.rs`. Tier 2 in
  `crates/version_0_9`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports; section 4 lists the plugin commands and the metadata.
