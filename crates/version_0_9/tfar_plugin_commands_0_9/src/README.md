# Plugin commands for 0.9 source

The source of `tfar_plugin_commands_0_9`: the plugin commands, the sender filter, the metadata
block and the error type.

## Contents

```text
crates/version_0_9/tfar_plugin_commands_0_9/src/
├── crate_role.rs      `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs           `Error`: unknown command, malformed field
├── lib.rs             the crate root: module header, `mod` lines and re-exports
├── metadata.rs        the `<TFAR>` block: status text, merge, read-back, playing test, info text
├── plugin_command.rs  `PluginCommand` with `encode` and `decode`, `TangentKind`, command names
├── prelude.rs         the command, filter and metadata names, with `RadioSubtype`
├── sender_filter.rs   `PLUGIN_NAME_PREFIX` and `accepts_sender`
└── tests/             unit tests: byte-exact texts, round trips, the filter, the metadata block
```

## How it works

```text
send:    PluginCommand ──encode──▶ text ──▶ sendPluginCommand(CURRENT_CHANNEL)
receive: onPluginCommandEvent(name, text) ──accepts_sender(name)──▶ decode(text) ──▶ PluginCommand
status:  format_metadata ──merge_metadata(own metadata)──▶ setClientSelfVariableAsString
others:  client metadata ──metadata_says_playing / info_data_text──▶ playing flag, info pane text
```

`plugin_command.rs` splits a received text with `tfar_arma_values::split_fields` and matches its
fields by slice pattern, so the field count is checked before any field is read; numbers go
through `tfar_arma_values`. `metadata.rs` finds the block from the first `<TFAR>` to the first
`</TFAR>` after it and never indexes: every cut is a checked `str::get`.

## Public surface

- `PluginCommand`, `TangentKind`, `plugin_command` (command names), `accepts_sender`,
  `PLUGIN_NAME_PREFIX`, `metadata` and its functions, `Error`, `Result`, `CRATE_ROLE`, `prelude`.

## Boundaries

- Depends on: `tfar_arma_values`, `thiserror`.
- Used by: `tfar_plugin_0_9`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list);
  tests live in `tests/` (`cargo xtask verify test-placement`); an encoded text changes only with
  the original's, held by the byte-exact tests.
