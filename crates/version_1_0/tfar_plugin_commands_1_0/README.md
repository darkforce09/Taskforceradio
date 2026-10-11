# Plugin commands for 1.0

The `tfar_plugin_commands_1_0` crate: the 1.0 plugin-to-plugin commands sent through TeamSpeak,
byte-compatible with the original, and the TFAR block of a client's metadata. Used by
`tfar_plugin_1_0`.

## How it works

Plugins of the same mod talk through TeamSpeak's `sendPluginCommand`, TAB-separated
(`reference/tfar_1_0/ts/src/plugin.cpp:1026-1084`):

| Message | Tokens | Sent | Source |
|---|---|---|---|
| `TANGENT[_LR\|_DD]\tPRESSED\|RELEASED\tfreq\trange\tsubtype[\tclassname]\tsw\tdd\tnick` | 8, or 9 with the class name | to the channel on a press or release | `CommandProcessor.cpp:326` |
| `RELEASE_ALL_TANGENTS\tnick` | 2 | to the channel | `CommandProcessor.cpp:378` |
| `VOLUME\tnick\tvoiceVolume\ttrue\|false` | 4 | to the channel when the talking flag changes | `plugin.cpp:841` |
| `REQVOL\tclientId` | 2 | never; answered with `VOLUME` | `plugin.cpp:1056-1064` |

`decode` recognises a command by name and token count together, as the original does, and reads
a tangent's fields by position relative to its count: 8 tokens are sw 5, dd 6, nick 7; 9 tokens
are classname 5, sw 6, dd 7, nick 8 (the original reads the flags from 6 and 7 in both forms).
`encode` writes the original's bytes, the tangent's range as the game sent it, so decoding and
encoding gives the same text. `accepts_sender` keeps only commands from a plugin whose name starts
with `TFAR`.

The metadata block is published every 4 s between `<TFAR>` and `</TFAR>` in the client's
TeamSpeak metadata:

```text
Connected to Game: [B]Yes|No[/B]
Playing: [B]Yes|No[/B]
Plugin version: [B]<plugin version>[/B]
Addon version: [B]<addon version>[/B]
```

`format_metadata` gives that block, `merge_metadata` puts it into the existing metadata (or
removes it when empty), `metadata_says_playing` is the plugin-enabled check other clients run,
and `info_data_text` is the info panel's text, cut to the 512-byte buffer.

`command_codec.rs` matches the split tokens against each command's name and token count in one
slice pattern per command, so a count the original does not accept is `Error::Unrecognised`, and
reads the tangent's tail (`[sw, dd, nick]` or `[classname, sw, dd, nick]`) by its own layout.
`metadata.rs` works on whole metadata strings with `str::find` for the first tags, as the
original's `std::string::find`.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_plugin_commands_1_0  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

- `PluginCommand`, `TangentBroadcast`, `TangentKind`, `TeamspeakClientId`; `encode`, `decode`.
- `PLUGIN_NAME_PREFIX`, `accepts_sender`.
- `format_metadata`, `merge_metadata`, `metadata_block`, `metadata_says_playing`,
  `info_data_text`, `METADATA_START_TAG`, `METADATA_END_TAG`, `DISABLED_METADATA`,
  `PLAYING_METADATA_PREFIX`, `INFO_DATA_BUFFER_SIZE`.
- `Error` (`Unrecognised`, whose message is the original's `UNKNOWN PLUGIN COMMAND` log line, and
  `Malformed`), `Result`, `prelude`.

## Boundaries

- Depends on: `tfar_arma_values` (`RadioSubtype`, `parse_int`); `thiserror` for the error type.
- Used by: `tfar_plugin_1_0`.
- Rules: the encoded bytes match what the original 1.0 plugin sends
  (`every_command_round_trips_byte_for_byte`, `the_status_block_is_the_original_text`); an 8-token
  tangent reads its flags from tokens 5 and 6
  (`an_eight_token_tangent_reads_its_flags_from_tokens_five_and_six`). Tier 2 in
  `crates/version_1_0`: every workspace dependency has a lower tier and follows the category edge matrix (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/behaviour_specs/tfar_1_0_plugin.md) — §4 the
  client-to-client protocol and the metadata.
- [Bug fixes](/documentation/architecture/bug_fixes.md) — the original bugs this crate fixes.
