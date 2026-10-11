//! `PluginSettings`: the twenty settings of the 1.0 plugin, `SETCFG` applied to them, and the
//! refresh flag the extension answers `DFRAME` with.
//!
//! **Role:** hold every setting with its typed value, apply one `SETCFG` message with the
//! original's key matching and conversions, and track whether the game must send its settings
//! again (`NEEDCFG`).
//!
//! **Position:** a module of `tfar_plugin_logic_1_0::plugin_settings`; built on `SettingKey`,
//! `SettingValue` and `TypeTag`; owned by `tfar_plugin_1_0`, read by the client state, routing and
//! voice pipeline crates.
//!
//! **Signals and state:** the settings themselves; the type has no interior mutability, so the
//! plugin that owns it decides which thread writes (the async command worker) and how readers see
//! it (a lock or a snapshot).
//!
//! **Invariants:** every setting keeps the kind of its default; a failed set changes nothing,
//! the refresh flag included; the refresh flag starts true, any successful set clears it, and only
//! [`PluginSettings::mark_needs_refresh`] sets it again (`reference/tfar_1_0/ts/src/settings.hpp:
//! 149-188`, `:215`).

use core::time::Duration;

use crate::plugin_settings::error::{Error, Result};
use crate::plugin_settings::setting_key::SettingKey;
use crate::plugin_settings::setting_value::SettingValue;
use crate::plugin_settings::type_tag::TypeTag;

#[cfg(test)]
#[path = "tests/settings_table.rs"]
mod tests;

/// What one successful [`PluginSettings::set`] did: the setting it wrote. It is reported on every
/// successful set, the value new or not, as the original's `configValueSet` signal fires.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SettingChanged {
    /// The setting that was written.
    pub key: SettingKey,
}

/// Every setting of the 1.0 plugin with its current value, and the refresh flag.
#[derive(Clone, Debug, PartialEq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each flag is one independent setting of the original, not a hidden state machine"
)]
pub struct PluginSettings {
    full_duplex: bool,
    addon_version: String,
    serious_channel_name: String,
    serious_channel_password: String,
    intercom_volume: f32,
    intercom_enabled: bool,
    plugin_timeout_seconds: f32,
    headset_lowered: bool,
    spectator_not_hear_enemies: bool,
    spectator_can_hear_friendlies: bool,
    tangent_release_delay_milliseconds: f32,
    move_while_tabbed_out: bool,
    intercom_ducking: f32,
    minimum_plugin_version: f32,
    object_interception_strength: f32,
    voice_cone: bool,
    allow_debugging: bool,
    no_automove_spectator: bool,
    disable_automatic_mute: bool,
    mute_spectators: bool,
    needs_refresh: bool,
}

impl Default for PluginSettings {
    /// Every setting at its default (the table in `reference/tfar_1_0/ts/src/settings.hpp:11-31`),
    /// and the refresh flag set.
    fn default() -> Self {
        Self {
            full_duplex: true,
            addon_version: "unknown".to_owned(),
            serious_channel_name: String::new(),
            serious_channel_password: String::new(),
            intercom_volume: 0.1,
            intercom_enabled: true,
            plugin_timeout_seconds: 4.0,
            headset_lowered: false,
            spectator_not_hear_enemies: false,
            spectator_can_hear_friendlies: true,
            tangent_release_delay_milliseconds: 0.0,
            move_while_tabbed_out: false,
            intercom_ducking: 0.2,
            minimum_plugin_version: 0.0,
            object_interception_strength: 400.0,
            voice_cone: true,
            allow_debugging: true,
            no_automove_spectator: false,
            disable_automatic_mute: false,
            mute_spectators: false,
            needs_refresh: true,
        }
    }
}

impl PluginSettings {
    /// Applies one `SETCFG` message (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:388-414`).
    ///
    /// - `key_text` is matched by [`SettingKey::from_key_text`] (a case-insensitive prefix).
    /// - `type_tag` is the message's fourth field. `None` stands for a message whose token count is
    ///   not 4: the original then sets the empty text instead of `value_text`, and so does this.
    /// - The value read under the tag is converted into the setting's kind
    ///   ([`SettingValue::converted_to`]).
    ///
    /// On success the refresh flag clears.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownKey`] when no setting matches (its message is the original's text),
    /// [`Error::MalformedNumber`] for a `SCALAR` value that is not a number, and
    /// [`Error::TextForTypedSetting`] for a text set on a boolean or number setting. Every error
    /// leaves the settings and the refresh flag unchanged.
    pub fn set(
        &mut self,
        key_text: &str,
        value_text: &str,
        type_tag: Option<TypeTag>,
    ) -> Result<SettingChanged> {
        let key = SettingKey::from_key_text(key_text).ok_or_else(|| Error::UnknownKey {
            key: key_text.to_owned(),
        })?;
        let (incoming, sent_text) = match type_tag {
            Some(tag) => (tag.read_value(value_text)?, value_text),
            None => (SettingValue::Text(String::new()), ""),
        };
        self.store(key, incoming, sent_text)?;
        self.needs_refresh = false;
        Ok(SettingChanged { key })
    }

    /// Writes `incoming`, converted into the kind of `key`, into the setting; `sent_text` is the
    /// text the value came from, for the error.
    fn store(&mut self, key: SettingKey, incoming: SettingValue, sent_text: &str) -> Result<()> {
        let text_error = || Error::TextForTypedSetting {
            key,
            value: sent_text.to_owned(),
        };
        let flag = |value: SettingValue| value.into_bool().ok_or_else(text_error);
        let number = |value: SettingValue| value.into_number().ok_or_else(text_error);
        match key {
            SettingKey::FullDuplex => self.full_duplex = flag(incoming)?,
            SettingKey::AddonVersion => self.addon_version = incoming.into_text(),
            SettingKey::SeriousChannelName => self.serious_channel_name = incoming.into_text(),
            SettingKey::SeriousChannelPassword => {
                self.serious_channel_password = incoming.into_text();
            }
            SettingKey::IntercomVolume => self.intercom_volume = number(incoming)?,
            SettingKey::IntercomEnabled => self.intercom_enabled = flag(incoming)?,
            SettingKey::PluginTimeout => self.plugin_timeout_seconds = number(incoming)?,
            SettingKey::HeadsetLowered => self.headset_lowered = flag(incoming)?,
            SettingKey::SpectatorNotHearEnemies => {
                self.spectator_not_hear_enemies = flag(incoming)?;
            }
            SettingKey::SpectatorCanHearFriendlies => {
                self.spectator_can_hear_friendlies = flag(incoming)?;
            }
            SettingKey::TangentReleaseDelay => {
                self.tangent_release_delay_milliseconds = number(incoming)?;
            }
            SettingKey::MoveWhileTabbedOut => self.move_while_tabbed_out = flag(incoming)?,
            SettingKey::IntercomDucking => self.intercom_ducking = number(incoming)?,
            SettingKey::MinimumPluginVersion => self.minimum_plugin_version = number(incoming)?,
            SettingKey::ObjectInterceptionStrength => {
                self.object_interception_strength = number(incoming)?;
            }
            SettingKey::VoiceCone => self.voice_cone = flag(incoming)?,
            SettingKey::AllowDebugging => self.allow_debugging = flag(incoming)?,
            SettingKey::NoAutomoveSpectator => self.no_automove_spectator = flag(incoming)?,
            SettingKey::DisableAutomaticMute => self.disable_automatic_mute = flag(incoming)?,
            SettingKey::MuteSpectators => self.mute_spectators = flag(incoming)?,
        }
        Ok(())
    }

    /// The current value of `key`, for diagnostics and generic readers.
    pub fn value(&self, key: SettingKey) -> SettingValue {
        match key {
            SettingKey::FullDuplex => SettingValue::Bool(self.full_duplex),
            SettingKey::AddonVersion => SettingValue::Text(self.addon_version.clone()),
            SettingKey::SeriousChannelName => SettingValue::Text(self.serious_channel_name.clone()),
            SettingKey::SeriousChannelPassword => {
                SettingValue::Text(self.serious_channel_password.clone())
            }
            SettingKey::IntercomVolume => SettingValue::Number(self.intercom_volume),
            SettingKey::IntercomEnabled => SettingValue::Bool(self.intercom_enabled),
            SettingKey::PluginTimeout => SettingValue::Number(self.plugin_timeout_seconds),
            SettingKey::HeadsetLowered => SettingValue::Bool(self.headset_lowered),
            SettingKey::SpectatorNotHearEnemies => {
                SettingValue::Bool(self.spectator_not_hear_enemies)
            }
            SettingKey::SpectatorCanHearFriendlies => {
                SettingValue::Bool(self.spectator_can_hear_friendlies)
            }
            SettingKey::TangentReleaseDelay => {
                SettingValue::Number(self.tangent_release_delay_milliseconds)
            }
            SettingKey::MoveWhileTabbedOut => SettingValue::Bool(self.move_while_tabbed_out),
            SettingKey::IntercomDucking => SettingValue::Number(self.intercom_ducking),
            SettingKey::MinimumPluginVersion => SettingValue::Number(self.minimum_plugin_version),
            SettingKey::ObjectInterceptionStrength => {
                SettingValue::Number(self.object_interception_strength)
            }
            SettingKey::VoiceCone => SettingValue::Bool(self.voice_cone),
            SettingKey::AllowDebugging => SettingValue::Bool(self.allow_debugging),
            SettingKey::NoAutomoveSpectator => SettingValue::Bool(self.no_automove_spectator),
            SettingKey::DisableAutomaticMute => SettingValue::Bool(self.disable_automatic_mute),
            SettingKey::MuteSpectators => SettingValue::Bool(self.mute_spectators),
        }
    }

    /// Whether the game must send its settings again: the extension answers `DFRAME` with
    /// `NEEDCFG` while this holds, else `OK`. True at start, false after any successful set.
    pub const fn needs_refresh(&self) -> bool {
        self.needs_refresh
    }

    /// Sets the refresh flag, as the plugin does when the game disconnects in game
    /// (`reference/tfar_1_0/ts/src/task_force_radio.cpp:24`).
    pub const fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
    }

    /// `full_duplex`: a radio receives while its owner transmits.
    pub const fn full_duplex(&self) -> bool {
        self.full_duplex
    }

    /// `addon_version`: the addon version the game reports.
    pub fn addon_version(&self) -> &str {
        &self.addon_version
    }

    /// `serious_channelName`: the channel serious mode moves into; empty for none.
    pub fn serious_channel_name(&self) -> &str {
        &self.serious_channel_name
    }

    /// `serious_channelPassword`: the serious channel's password.
    pub fn serious_channel_password(&self) -> &str {
        &self.serious_channel_password
    }

    /// `intercomVolume`: the intercom volume.
    pub const fn intercom_volume(&self) -> f32 {
        self.intercom_volume
    }

    /// `intercomEnabled`: vehicle intercoms work.
    pub const fn intercom_enabled(&self) -> bool {
        self.intercom_enabled
    }

    /// `pluginTimeout`: how long the game may stay silent before it counts as gone. The seconds
    /// are cut to whole milliseconds as `duration_cast<milliseconds>` cuts them
    /// (`reference/tfar_1_0/ts/src/SharedMemoryHandler.cpp:229`); a negative value is zero.
    pub fn plugin_timeout(&self) -> Duration {
        whole_milliseconds(self.plugin_timeout_seconds * 1000.0)
    }

    /// `headsetLowered`: the player has lowered the headset.
    pub const fn headset_lowered(&self) -> bool {
        self.headset_lowered
    }

    /// `spectatorNotHearEnemies`: a spectator does not hear enemy players.
    pub const fn spectator_not_hear_enemies(&self) -> bool {
        self.spectator_not_hear_enemies
    }

    /// `spectatorCanHearFriendlies`: a spectator hears friendly players.
    pub const fn spectator_can_hear_friendlies(&self) -> bool {
        self.spectator_can_hear_friendlies
    }

    /// `tangentReleaseDelay`: how long a transmission continues after its release, cut to whole
    /// milliseconds as `static_cast<int>` cuts it (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:
    /// 364`); a negative value is zero, as the original then does not wait.
    pub fn tangent_release_delay(&self) -> Duration {
        whole_milliseconds(self.tangent_release_delay_milliseconds)
    }

    /// `moveWhileTabbedOut`: the serious channel move also runs while Arma is not in front.
    pub const fn move_while_tabbed_out(&self) -> bool {
        self.move_while_tabbed_out
    }

    /// `intercomDucking`: how much the intercom ducks under radio traffic.
    pub const fn intercom_ducking(&self) -> f32 {
        self.intercom_ducking
    }

    /// `minimumPluginVersion`: the lowest plugin build number the addon accepts.
    pub const fn minimum_plugin_version(&self) -> f32 {
        self.minimum_plugin_version
    }

    /// `objectInterceptionStrength`: the strength of object interception.
    pub const fn object_interception_strength(&self) -> f32 {
        self.object_interception_strength
    }

    /// `voiceCone`: direct speech is louder in front of the speaker.
    pub const fn voice_cone(&self) -> bool {
        self.voice_cone
    }

    /// `allowDebugging`: the `/tfar` debug commands work.
    pub const fn allow_debugging(&self) -> bool {
        self.allow_debugging
    }

    /// `noAutomoveSpectator`: a spectator is not moved into the serious channel.
    pub const fn no_automove_spectator(&self) -> bool {
        self.no_automove_spectator
    }

    /// `disableAutomaticMute`: serious mode never mutes players automatically.
    pub const fn disable_automatic_mute(&self) -> bool {
        self.disable_automatic_mute
    }

    /// `muteSpectators`: spectators are muted for the living.
    pub const fn mute_spectators(&self) -> bool {
        self.mute_spectators
    }
}

/// `milliseconds` cut toward zero to a whole number of milliseconds; zero for a negative value.
fn whole_milliseconds(milliseconds: f32) -> Duration {
    let whole = milliseconds.trunc();
    if whole.is_nan() || whole <= 0.0 {
        return Duration::ZERO;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the value is integral and positive, and a float-to-integer cast saturates at u64::MAX"
    )]
    let whole_milliseconds = whole as u64;
    Duration::from_millis(whole_milliseconds)
}
