# TeamSpeak settings database

The `tfar_ts3_settings_db` crate: reads TeamSpeak's settings.db for the values the plugins need,
today the push-to-talk delay of the default capture profile. Used by `tfar_plugin_0_9`,
`tfar_plugin_1_0`.

## How it works

A plugin's `ts3plugin_init` asks TeamSpeak for its config folder, builds the database path with
`settings_database_path` and calls `read_push_to_talk_delay`. The function opens `settings.db`
read-only with `rusqlite` (TeamSpeak holds the file open), checks the `Profiles` table, reads the
`value` of the `Capture/Default/PreProcessing` row and parses it line by line:
`delay_ptt=true` turns the delay on, `delay_ptt_msecs=<n>` sets its length. The answer is a
`PushToTalkDelay`, whose `effective()` is the length when enabled and zero otherwise. A missing
file, table or row, a failed query and a malformed length are each a typed `Error`, which the
plugins read as zero delay.

`settings_database.rs` opens the file with `SQLITE_OPEN_READ_ONLY`, looks for the `Profiles`
table in `sqlite_master`, reads the `Capture/Default/PreProcessing` value (TEXT, BLOB, a number or
NULL, all as text) and hands it to `PushToTalkDelay::parse_pre_processing` in
`push_to_talk_delay.rs`, which reads one `key=value` per line, `\n` or `\r\n` ended. Each module
attaches its sibling test file in `tests/` with `#[path]`.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_ts3_settings_db  # the parser tests and the queries on temporary databases
```

## Configuration

`settings.db` in TeamSpeak's config folder (`~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/` for
the Flatpak), the `Profiles` row `Capture/Default/PreProcessing`: `delay_ptt` (default off) and
`delay_ptt_msecs` (default 0), read by `src/settings_database.rs` and `src/push_to_talk_delay.rs`.

## Public surface

- `read_push_to_talk_delay(settings_db)`: the delay from a `settings.db` file.
- `PushToTalkDelay` (`enabled`, `milliseconds`, `effective()`, `parse_pre_processing`).
- `settings_database_path(config_path)` and `SETTINGS_DATABASE_FILE_NAME`.
- `Error`, `Result`, `prelude`.

## Boundaries

- Depends on: no workspace crate; `rusqlite` (with SQLite bundled) for the query, `thiserror` for
  the error type; the `settings.db` file TeamSpeak writes.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`.
- Rules: the database is opened read-only and never created
  (`a_missing_file_is_a_typed_error_and_is_not_created`,
  `reading_leaves_the_database_bytes_unchanged`); a malformed length is an error, never a panic
  (`garbage_milliseconds_are_an_error`). Tier 0 in `crates/teamspeak`: every workspace dependency
  has a lower tier and follows the category edge matrix
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin and its settings.db query.
- [TFAR 1.0 plugin behaviour](/documentation/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin and its settings.db query.
