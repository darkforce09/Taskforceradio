# TeamSpeak crates

The boundary to the TeamSpeak 3 client: the plugin SDK export layer and the reader of TeamSpeak's
settings database.

## Contents

```text
crates/teamspeak/
├── tfar_ts3_host/         tier 0: the safe TeamSpeak export layer and function table
└── tfar_ts3_settings_db/  tier 0: TeamSpeak settings lookups
```

## How it works

`tfar_ts3_host` owns every `ts3plugin_*` export and the host function table, so the plugin apps
implement a safe trait instead of C exports. `tfar_ts3_settings_db` reads the settings TeamSpeak
keeps in SQLite.

## Getting started

Run from the repository root:

```bash
# the unit tests of every crate in this category
cargo test \
  -p tfar_ts3_host \
  -p tfar_ts3_settings_db
```

## Boundaries

- Depends on: `crates/foundation` and the teamspeak category itself.
- Used by: the plugin apps and `tools/tfar_ts3_mock`.
- Rules: `ts3plugin-sys` stays in `tfar_ts3_host` and the mock host; `rusqlite` stays in
  `tfar_ts3_settings_db`; `tfar_ts3_host` is on the unsafe allow-list
  (`cargo xtask verify crate-tiers` holds the edges, `cargo xtask verify dependency-firewalls` the
  external crates); each crate declares `category = "crates/teamspeak"` and its tier: 0 without a
  workspace dependency, else one above the highest.

## Related documentation

- [Library survey](/documentation/program/research/library_survey.md) — the external crates chosen for each need.
