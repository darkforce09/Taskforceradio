# Plugin settings for 1.0 source

The source of `tfar_plugin_settings_1_0`: the settings struct, the setting keys, the values and
their conversions, the type tags, the error type and the prelude.

## Contents

```text
crates/version_1_0/tfar_plugin_settings_1_0/src/
├── crate_role.rs       `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs            `Error`: unknown key, malformed number, text on a typed setting; `Result`
├── lib.rs              the crate root: module header, `mod` lines and re-exports
├── plugin_settings.rs  `PluginSettings` and `SettingChanged`: `SETCFG` applied, refresh flag, getters
├── prelude.rs          the settings, key, value and tag names for glob import
├── setting_key.rs      `SettingKey`: wire names, defaults, kinds and the prefix key match
├── setting_value.rs    `SettingValue` and `SettingKind`: a value and the cross-kind conversions
├── tests/              unit tests: key match, conversions, tags, every set path and the refresh flag
└── type_tag.rs         `TypeTag`: `BOOL`, `SCALAR` or text, and how each reads the value text
```

## How it works

`PluginSettings::set` finds the key with `SettingKey::from_key_text`, reads the value with
`TypeTag::read_value` (or takes the empty text when there is no tag), and converts it into the
setting's kind with `SettingValue::into_bool`, `into_number` or `into_text` before writing the
typed field; only then does the refresh flag clear. `SettingKey::default_value` is the single
table of defaults and kinds; `PluginSettings::default` holds the same values in its fields, and a
test holds the two equal.

## Public surface

- `PluginSettings`, `SettingChanged`, `SettingKey`, `SettingValue`, `SettingKind`, `TypeTag`,
  `Error`, `Result`, `CRATE_ROLE` and `prelude`, re-exported at the crate root.

## Boundaries

- Depends on: `thiserror`.
- Used by: `tfar_client_state_1_0`, `tfar_game_commands_1_0`, `tfar_plugin_1_0`,
  `tfar_radio_routing_1_0`, `tfar_voice_pipeline_1_0`.
- Rules: `PluginSettings::default` matches `SettingKey::default_value` for every key
  (`every_setting_starts_at_its_default`); production files hold no `unwrap`, `expect`, `panic` or
  indexing (clippy deny list); tests live in `tests/` (`cargo xtask verify test-placement`).
