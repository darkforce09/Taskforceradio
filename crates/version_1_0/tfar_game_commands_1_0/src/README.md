# Game commands for 1.0 source

The source of `tfar_game_commands_1_0`: the parser and its dispatch, the command names, the parsed
command, the replies, the field readers and the command payloads.

## Contents

```text
crates/version_1_0/tfar_game_commands_1_0/src/
├── command_name.rs  `CommandName`: the known names, exact matching and the async-only rule
├── commands/        the payload of each command and its parser
├── crate_role.rs    `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs         `Error`: wrong token count or malformed field, with the sync reply; `Result`
├── fields/          the field readers: tokens by position, booleans, coordinates, ids, vehicles
├── game_command.rs  `GameCommand`: one variant per command, its name and the fixed sync replies
├── lib.rs           the crate root: module header, `mod` lines and re-exports
├── parse.rs         `parse`: split, name lookup and dispatch to the command parsers
├── prelude.rs       the command, payload and parser names for glob import
├── replies.rs       the reply strings, the speaking codes and the bulk reply
└── tests/           unit tests: names, dispatch, simple commands, sync replies and reply strings
```

## How it works

```text
text ── parse ──► split at TAB (every field kept) ──► CommandName::from_wire_name
                                                          │ none ──► GameCommand::Unknown
                                                          ▼
                            MessageFields { command, tokens } ──► commands/<payload> parser
                                                          │
                                   GameCommand::<variant>(payload)  or  Error
```

`fields::MessageFields` wraps the tokens with the command they belong to, so every count check
and field read names the command and field in its error. The plugin answers a sync call with
`GameCommand::fixed_sync_reply` where no state is needed, from its state otherwise (the
`replies` builders), and with `Error::sync_reply` for a rejected message.

## Public surface

- `parse`, `GameCommand`, `CommandName`, the payload types of `commands/`, `VehicleDescriptor`,
  `NetworkObjectId`, `replies`, `Error`, `Result`, `CRATE_ROLE` and `prelude`.

## Boundaries

- Depends on: `tfar_arma_values`, `tfar_geometry`, `tfar_plugin_settings_1_0`, `thiserror`.
- Used by: `tfar_client_state_1_0`, `tfar_plugin_1_0`.
- Rules: only `parse.rs` maps names to parsers, and it covers every `CommandName`
  (`every_parsed_command_reports_its_name`); production files hold no `unwrap`, `expect`, `panic`
  or indexing (clippy deny list); tests live in `tests/` (`cargo xtask verify test-placement`).
