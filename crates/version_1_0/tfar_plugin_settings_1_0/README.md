# Plugin settings for 1.0

The `tfar_plugin_settings_1_0` crate: the 1.0 plugin's settings: the values `SETCFG` sets and the
defaults. Used by `tfar_client_state_1_0`, `tfar_game_commands_1_0`, `tfar_plugin_1_0`,
`tfar_radio_routing_1_0`, `tfar_voice_pipeline_1_0`.

## How it works

`PluginSettings` holds the twenty settings of `reference/tfar_1_0/ts/src/settings.hpp` as typed
fields, each starting at its default. `set(key_text, value_text, type_tag)` applies one `SETCFG`
message as `CommandProcessor.cpp:388-414` does:

- **Key:** the first setting, in the original's order, whose wire name equals the start of the key
  text in ASCII case-insensitive comparison (`_strnicmp` over the name's length, `settings.hpp:47`).
  The prefix rule is kept so every key the addon sends still matches; no wire name is a prefix of
  another, so the first match is the only one.
- **Value:** `BOOL` is true only for `true` or `TRUE`, `SCALAR` is a finite number, any other tag
  is text. `None` stands for a message whose token count is not 4, where the original sets the
  empty text.
- **Conversion:** the value takes the kind of the setting's default: a boolean from a number is
  `number > 0`, a number from a boolean 1 or 0, a text from a boolean `true`/`false` and from a
  number the six-decimal `%f` text. A text set on a boolean or number setting is an error (the
  original writes it through the wrong union member).

Every error leaves the settings unchanged. The refresh flag behind `NEEDCFG` starts true, clears on
any successful set and is set again by `mark_needs_refresh` when the game disconnects.

`PluginSettings::set` finds the key with `SettingKey::from_key_text`, reads the value with
`TypeTag::read_value` (or takes the empty text when there is no tag), and converts it into the
setting's kind with `SettingValue::into_bool`, `into_number` or `into_text` before writing the
typed field; only then does the refresh flag clear. `SettingKey::default_value` is the single
table of defaults and kinds; `PluginSettings::default` holds the same values in its fields, and a
test holds the two equal.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_plugin_settings_1_0  # the unit tests
```

## Configuration

The crate reads no setting itself; it holds the plugin's. The twenty settings, their defaults and
getters (`settings.hpp:12-31`):

| Key | Kind | Default | Getter |
|---|---|---|---|
| `full_duplex` | bool | true | `full_duplex()` |
| `addon_version` | text | `unknown` | `addon_version()` |
| `serious_channelName` | text | empty | `serious_channel_name()` |
| `serious_channelPassword` | text | empty | `serious_channel_password()` |
| `intercomVolume` | number | 0.1 | `intercom_volume()` |
| `intercomEnabled` | bool | true | `intercom_enabled()` |
| `pluginTimeout` | number (s) | 4 | `plugin_timeout() -> Duration` |
| `headsetLowered` | bool | false | `headset_lowered()` |
| `spectatorNotHearEnemies` | bool | false | `spectator_not_hear_enemies()` |
| `spectatorCanHearFriendlies` | bool | true | `spectator_can_hear_friendlies()` |
| `tangentReleaseDelay` | number (ms) | 0 | `tangent_release_delay() -> Duration` |
| `moveWhileTabbedOut` | bool | false | `move_while_tabbed_out()` |
| `intercomDucking` | number | 0.2 | `intercom_ducking()` |
| `minimumPluginVersion` | number | 0 | `minimum_plugin_version()` |
| `objectInterceptionStrength` | number | 400 | `object_interception_strength()` |
| `voiceCone` | bool | true | `voice_cone()` |
| `allowDebugging` | bool | true | `allow_debugging()` |
| `noAutomoveSpectator` | bool | false | `no_automove_spectator()` |
| `disableAutomaticMute` | bool | false | `disable_automatic_mute()` |
| `muteSpectators` | bool | false | `mute_spectators()` |

The two durations are cut to whole milliseconds and never negative.

## Public surface

- `PluginSettings`: `default()`, `set`, `value`, `needs_refresh`, `mark_needs_refresh` and one getter
  per setting.
- `SettingChanged`: the key a successful set wrote.
- `SettingKey`: `ALL`, `COUNT`, `wire_name`, `default_value`, `kind`, `from_key_text`.
- `SettingValue`, `SettingKind`: a value, its kind and the cross-kind conversions.
- `TypeTag`: `from_wire`, `read_value`.
- `Error` (`UnknownKey` with the original's message, `MalformedNumber`, `TextForTypedSetting`),
  `Result`, `prelude`.

## Boundaries

- Depends on: no workspace crate; `thiserror` for the error type.
- Used by: `tfar_client_state_1_0`, `tfar_game_commands_1_0`, `tfar_plugin_1_0`,
  `tfar_radio_routing_1_0`, `tfar_voice_pipeline_1_0`.
- Rules: an unknown key or a malformed value is an `Error` and leaves the settings unchanged
  (`a_failed_set_changes_nothing_and_keeps_the_refresh_flag`); the defaults and key order are the
  original's (`the_defaults_are_the_original_table`). Tier 0 in `crates/version_1_0`: every
  workspace dependency has a lower tier and follows the category edge matrix
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/behaviour_specs/tfar_1_0_plugin.md) — §2.2
  `SETCFG` and `NEEDCFG`, §5 the settings table.
