//! The apply functions of the plugin commands other than the tangent: `VOLUME` and
//! `RELEASE_ALL_TANGENTS`.

use std::time::Instant;

use tfar_plugin_commands_1_0::TeamspeakClientId;

use crate::session::session_state::SessionState;

#[cfg(test)]
#[path = "tests/remote_commands.rs"]
mod tests;

/// What [`SessionState::apply_remote_volume`] changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VolumeApplied {
    /// The sender.
    pub client_id: TeamspeakClientId,
    /// The sender is the local player; otherwise the app recomputes the sender's mute.
    pub is_own_client: bool,
}

/// What [`SessionState::apply_release_all_tangents`] changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReleaseAllApplied {
    /// The sender.
    pub client_id: TeamspeakClientId,
    /// The local player stopped receiving the sender.
    pub is_receiving_changed: bool,
}

impl SessionState {
    /// Takes a `VOLUME` plugin command at `now` (`plugin.cpp:1041-1057`): the sender's voice
    /// range and talk flag, and the sender runs TFAR. `None` for an unknown nickname.
    pub fn apply_remote_volume(
        &mut self,
        nickname: &str,
        voice_volume: i32,
        is_talking: bool,
        now: Instant,
    ) -> Option<VolumeApplied> {
        let own_client_id = self.clients.own_client_id();
        let client = self.clients.get_mut(nickname)?;
        client.voice_volume = voice_volume;
        client.mark_plugin_enabled(now);
        client.is_talking_now = is_talking;
        Some(VolumeApplied {
            client_id: client.client_id(),
            is_own_client: own_client_id == Some(client.client_id()),
        })
    }

    /// Takes a `RELEASE_ALL_TANGENTS` plugin command (`processAllTangentRelease`,
    /// `plugin.cpp:868-876`): the sender holds no tangent any more, so its range is 0, and the
    /// local player stops receiving it. `None` for an unknown nickname.
    pub fn apply_release_all_tangents(&mut self, nickname: &str) -> Option<ReleaseAllApplied> {
        let client = self.clients.get_mut(nickname)?;
        client.transmitting_over = None;
        client.transmitting_frequency = None;
        client.range = 0;
        let client_id = client.client_id();
        let is_receiving_changed = self
            .clients
            .own_mut()
            .is_some_and(|own| own.receiving.stop(client_id));
        Some(ReleaseAllApplied {
            client_id,
            is_receiving_changed,
        })
    }
}
