# Plugin commands for 1.0 source

The source of `tfar_plugin_commands_1_0`: the plugin command types, their codec, the sender
filter, the metadata block, the error type and the prelude.

## Contents

```text
crates/version_1_0/tfar_plugin_commands_1_0/src/
├── command_codec.rs   `encode` and `decode`: the TAB-separated text, byte for byte
├── crate_role.rs      `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs           `Error`: unrecognised command or malformed field; `Result`
├── lib.rs             the crate root: module header, `mod` lines and re-exports
├── metadata.rs        the TFAR metadata block: format, merge, read, playing check, info text
├── plugin_command.rs  `PluginCommand`, `TangentBroadcast`, `TangentKind`, `TeamspeakClientId`
├── prelude.rs         the command, codec, filter and metadata names for glob import
├── sender_filter.rs   `PLUGIN_NAME_PREFIX` and `accepts_sender`
└── tests/             unit tests: byte-exact texts, round trips, both tangent forms, filter, metadata
```

## How it works

`command_codec.rs` matches the split tokens against each command's name and token count in one
slice pattern per command, so a count the original does not accept is `Error::Unrecognised`, and
reads the tangent's tail (`[sw, dd, nick]` or `[classname, sw, dd, nick]`) by its own layout.
`metadata.rs` works on whole metadata strings with `str::find` for the first tags, as the
original's `std::string::find`.

## Public surface

- `PluginCommand`, `TangentBroadcast`, `TangentKind`, `TeamspeakClientId`, `encode`, `decode`,
  `PLUGIN_NAME_PREFIX`, `accepts_sender`, the metadata functions and constants, `Error`, `Result`,
  `CRATE_ROLE` and `prelude`, re-exported at the crate root.

## Boundaries

- Depends on: `tfar_arma_values`, `thiserror`.
- Used by: `tfar_plugin_1_0`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list);
  tests live in `tests/` (`cargo xtask verify test-placement`).
