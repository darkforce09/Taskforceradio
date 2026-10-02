# TeamSpeak settings database source

The source of `tfar_ts3_settings_db`: the read-only query on `settings.db`, the parser of the
capture pre-processing profile, the error type and the prelude.

## Contents

```text
crates/teamspeak/tfar_ts3_settings_db/src/
├── crate_role.rs          `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs               `Error`, the `thiserror` enum of every failure, and `Result`
├── lib.rs                 the crate root: module header, `mod` lines and re-exports
├── prelude.rs             the public names for glob import, without `Error` and `Result`
├── push_to_talk_delay.rs  `PushToTalkDelay` and the parser of the profile text
├── settings_database.rs   `read_push_to_talk_delay`: the read-only open and the query
└── tests/                 the parser tests and the queries on temporary databases
```

## How it works

`settings_database.rs` opens the file with `SQLITE_OPEN_READ_ONLY`, looks for the `Profiles`
table in `sqlite_master`, reads the `Capture/Default/PreProcessing` value (TEXT, BLOB, a number or
NULL, all as text) and hands it to `PushToTalkDelay::parse_pre_processing` in
`push_to_talk_delay.rs`, which reads one `key=value` per line, `\n` or `\r\n` ended. Each module
attaches its sibling test file in `tests/` with `#[path]`.

## Public surface

- `read_push_to_talk_delay`, `settings_database_path`, `SETTINGS_DATABASE_FILE_NAME`.
- `PushToTalkDelay`.
- `Error`, `Result`, `CRATE_ROLE` and `prelude`.

## Boundaries

- Depends on: `rusqlite`, `thiserror`.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`.
- Rules: the query is the one of `reference/tfar_1_0/ts/src/plugin.cpp` (line 351) and its
  line rules those of the original `pttCallback` (`tests/push_to_talk_delay.rs`); production files
  hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests live in `tests/`
  (`cargo xtask verify test-placement`).
