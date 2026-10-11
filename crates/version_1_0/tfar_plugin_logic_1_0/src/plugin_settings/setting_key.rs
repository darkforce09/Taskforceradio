//! `SettingKey`: the twenty settings of the 1.0 plugin and the rule that matches a `SETCFG` key.
//!
//! **Role:** name every setting by its wire name, give its default, and find the setting a key
//! text from the game names.
//!
//! **Position:** a module of `tfar_plugin_logic_1_0::plugin_settings`; used by `PluginSettings::set` and
//! by every caller that reads or reports one setting.
//!
//! **Signals and state:** none; a `Copy` enum and pure functions, callable from any thread.
//!
//! **Invariants:** [`SettingKey::ALL`] lists the settings in the order of the `Settings` macro of
//! `reference/tfar_1_0/ts/src/settings.hpp:11-31`, and a key text matches the first setting in
//! that order whose wire name is a case-insensitive prefix of it (`_strnicmp` over the name's
//! length, `settings.hpp:47-54`).

use crate::plugin_settings::setting_value::{SettingKind, SettingValue};

#[cfg(test)]
#[path = "tests/setting_key.rs"]
mod tests;

/// One setting of the 1.0 plugin, as `SETCFG` names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SettingKey {
    /// `full_duplex`: a radio receives while its owner transmits; default `true`.
    FullDuplex,
    /// `addon_version`: the addon version the game reports; default `"unknown"`.
    AddonVersion,
    /// `serious_channelName`: the channel serious mode moves into; default `""`.
    SeriousChannelName,
    /// `serious_channelPassword`: that channel's password; default `""`.
    SeriousChannelPassword,
    /// `intercomVolume`: the intercom volume; default `0.1`.
    IntercomVolume,
    /// `intercomEnabled`: vehicle intercoms work; default `true`.
    IntercomEnabled,
    /// `pluginTimeout`: seconds without a game message before the game counts as gone; default `4`.
    PluginTimeout,
    /// `headsetLowered`: the player has lowered the headset; default `false`.
    HeadsetLowered,
    /// `spectatorNotHearEnemies`: a spectator does not hear enemy players; default `false`.
    SpectatorNotHearEnemies,
    /// `spectatorCanHearFriendlies`: a spectator hears friendly players; default `true`.
    SpectatorCanHearFriendlies,
    /// `tangentReleaseDelay`: milliseconds the transmission continues after a release; default `0`.
    TangentReleaseDelay,
    /// `moveWhileTabbedOut`: the serious channel move also runs while Arma is not in front;
    /// default `false`.
    MoveWhileTabbedOut,
    /// `intercomDucking`: how much the intercom ducks under radio traffic; default `0.2`.
    IntercomDucking,
    /// `minimumPluginVersion`: the lowest plugin build number the addon accepts; default `0`.
    MinimumPluginVersion,
    /// `objectInterceptionStrength`: the strength of object interception; default `400`.
    ObjectInterceptionStrength,
    /// `voiceCone`: direct speech is louder in front of the speaker; default `true`.
    VoiceCone,
    /// `allowDebugging`: the `/tfar` debug commands work; default `true`.
    AllowDebugging,
    /// `noAutomoveSpectator`: a spectator is not moved into the serious channel; default `false`.
    NoAutomoveSpectator,
    /// `disableAutomaticMute`: serious mode never mutes players automatically; default `false`.
    DisableAutomaticMute,
    /// `muteSpectators`: spectators are muted for the living; default `false`.
    MuteSpectators,
}

impl SettingKey {
    /// The number of settings.
    pub const COUNT: usize = 20;

    /// Every setting, in the original's order; key matching tries them in this order.
    pub const ALL: [Self; Self::COUNT] = [
        Self::FullDuplex,
        Self::AddonVersion,
        Self::SeriousChannelName,
        Self::SeriousChannelPassword,
        Self::IntercomVolume,
        Self::IntercomEnabled,
        Self::PluginTimeout,
        Self::HeadsetLowered,
        Self::SpectatorNotHearEnemies,
        Self::SpectatorCanHearFriendlies,
        Self::TangentReleaseDelay,
        Self::MoveWhileTabbedOut,
        Self::IntercomDucking,
        Self::MinimumPluginVersion,
        Self::ObjectInterceptionStrength,
        Self::VoiceCone,
        Self::AllowDebugging,
        Self::NoAutomoveSpectator,
        Self::DisableAutomaticMute,
        Self::MuteSpectators,
    ];

    /// The setting's wire name, as the addon writes it in `SETCFG`
    /// (`reference/tfar_1_0/addons/core/functions/plugin/fnc_sendPluginConfig.sqf`).
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::FullDuplex => "full_duplex",
            Self::AddonVersion => "addon_version",
            Self::SeriousChannelName => "serious_channelName",
            Self::SeriousChannelPassword => "serious_channelPassword",
            Self::IntercomVolume => "intercomVolume",
            Self::IntercomEnabled => "intercomEnabled",
            Self::PluginTimeout => "pluginTimeout",
            Self::HeadsetLowered => "headsetLowered",
            Self::SpectatorNotHearEnemies => "spectatorNotHearEnemies",
            Self::SpectatorCanHearFriendlies => "spectatorCanHearFriendlies",
            Self::TangentReleaseDelay => "tangentReleaseDelay",
            Self::MoveWhileTabbedOut => "moveWhileTabbedOut",
            Self::IntercomDucking => "intercomDucking",
            Self::MinimumPluginVersion => "minimumPluginVersion",
            Self::ObjectInterceptionStrength => "objectInterceptionStrength",
            Self::VoiceCone => "voiceCone",
            Self::AllowDebugging => "allowDebugging",
            Self::NoAutomoveSpectator => "noAutomoveSpectator",
            Self::DisableAutomaticMute => "disableAutomaticMute",
            Self::MuteSpectators => "muteSpectators",
        }
    }

    /// The setting's default value; its kind is the setting's kind for good.
    pub fn default_value(self) -> SettingValue {
        match self {
            Self::FullDuplex
            | Self::IntercomEnabled
            | Self::SpectatorCanHearFriendlies
            | Self::VoiceCone
            | Self::AllowDebugging => SettingValue::Bool(true),
            Self::HeadsetLowered
            | Self::SpectatorNotHearEnemies
            | Self::MoveWhileTabbedOut
            | Self::NoAutomoveSpectator
            | Self::DisableAutomaticMute
            | Self::MuteSpectators => SettingValue::Bool(false),
            Self::AddonVersion => SettingValue::Text("unknown".to_owned()),
            Self::SeriousChannelName | Self::SeriousChannelPassword => {
                SettingValue::Text(String::new())
            }
            Self::IntercomVolume => SettingValue::Number(0.1),
            Self::PluginTimeout => SettingValue::Number(4.0),
            Self::TangentReleaseDelay | Self::MinimumPluginVersion => SettingValue::Number(0.0),
            Self::IntercomDucking => SettingValue::Number(0.2),
            Self::ObjectInterceptionStrength => SettingValue::Number(400.0),
        }
    }

    /// The kind of value the setting holds: the kind of its default.
    pub fn kind(self) -> SettingKind {
        self.default_value().kind()
    }

    /// The setting a `SETCFG` key text names, by the original rule: the first setting, in
    /// [`SettingKey::ALL`] order, whose wire name equals the start of `key_text` in ASCII letter
    /// case-insensitive comparison. `key_text` may run on past the name (`full_duplexX` is
    /// `full_duplex`), but never stop short of it. `None` when no setting matches.
    ///
    /// The prefix rule is kept so every key the addon sends still matches as it did.
    pub fn from_key_text(key_text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|key| {
            let name = key.wire_name().as_bytes();
            key_text
                .as_bytes()
                .get(..name.len())
                .is_some_and(|start| start.eq_ignore_ascii_case(name))
        })
    }
}
