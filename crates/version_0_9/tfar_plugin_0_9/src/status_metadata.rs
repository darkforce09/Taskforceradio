//! The plugin's status in the local player's client metadata, which the other players' plugins
//! read to decide whether this player runs TFAR.

use tfar_plugin_commands_0_9::{DISABLED_METADATA, format_metadata, merge_metadata};
use tfar_ts3_host::ServerConnectionHandlerId;

use crate::plugin_core::PluginCore;

#[cfg(test)]
#[path = "tests/status_metadata.rs"]
mod tests;

/// The plugin version the metadata names (`PLUGIN_VERSION`,
/// `reference/tfar_0_9_12/ts/src/plugin.cpp:69`).
pub const PLUGIN_VERSION: &str = "0.9.12";

/// `existing` metadata with its `<TFAR>` block set to `status`; `None` when nothing changes.
pub fn metadata_with_status(existing: &str, status: &str) -> Option<String> {
    let merged = merge_metadata(existing, status);
    (merged != existing).then_some(merged)
}

impl PluginCore {
    /// Writes the current status (link connected, in game, versions) into the local player's
    /// metadata on `server` (`updateUserStatusInfo(true)`); a dormant plugin writes nothing.
    pub fn publish_status_metadata(&self, server: ServerConnectionHandlerId) {
        if self.is_dormant() {
            return;
        }
        let addon_version = self
            .with_state(server, |state| state.radio.addon_version.clone())
            .unwrap_or_default();
        let status = format_metadata(
            self.is_link_connected(),
            self.is_in_game(),
            PLUGIN_VERSION,
            &addon_version,
        );
        self.write_status(server, &status);
    }

    /// Writes the disabled status into the local player's metadata on `server`
    /// (`updateUserStatusInfo(false)` at shutdown); a dormant plugin writes nothing.
    pub fn publish_disabled_metadata(&self, server: ServerConnectionHandlerId) {
        if !self.is_dormant() {
            self.write_status(server, DISABLED_METADATA);
        }
    }

    /// Sets the `<TFAR>` block of our metadata to `status`, keeping what surrounds it.
    fn write_status(&self, server: ServerConnectionHandlerId, status: &str) {
        let team_speak = self.team_speak();
        let Some(me) = team_speak.my_client_id(server) else {
            return;
        };
        let existing = team_speak.client_metadata(server, me).unwrap_or_default();
        if let Some(metadata) = metadata_with_status(&existing, status)
            && let Err(error) = team_speak.set_own_metadata(server, &metadata)
        {
            tracing::warn!(%error, "the plugin status could not be written to the metadata");
        }
    }
}
