//! `VERSION`, `SPEAKERS` and `KILLED`: the handshake, the loudspeaker radios and a death.

use tfar_game_commands_0_9::SpeakerRecord;

use crate::server_radio_data::{INVALID_DATA_FRAME, ServerRadioData};
use crate::speaker_radio::SpeakerRadio;

#[cfg(test)]
#[path = "tests/mission_messages.rs"]
mod tests;

impl ServerRadioData {
    /// Stores the `VERSION` handshake and moves to the next data frame
    /// (`reference/tfar_0_9_12/ts/src/plugin.cpp:1789-1798`).
    pub fn apply_version(
        &mut self,
        addon_version: &str,
        serious_channel_name: &str,
        serious_channel_password: &str,
    ) {
        addon_version.clone_into(&mut self.addon_version);
        serious_channel_name.clone_into(&mut self.serious_channel_name);
        serious_channel_password.clone_into(&mut self.serious_channel_password);
        self.current_data_frame = self.current_data_frame.saturating_add(1);
    }

    /// Replaces the loudspeaker radios with `records`, filing each under every frequency it
    /// lists (`processSpeakers`, plugin.cpp:1728); an empty list clears them.
    pub fn apply_speakers(&mut self, records: &[SpeakerRecord]) {
        self.speakers.clear();
        for record in records {
            let radio = SpeakerRadio::from_record(record);
            for frequency in &record.frequencies {
                self.speakers
                    .entry(frequency.clone())
                    .or_default()
                    .push(radio.clone());
            }
        }
    }

    /// Marks the player `nickname` dead: their data frame becomes invalid, so their data stops
    /// being current (`processUnitKilled`, plugin.cpp:1562). Returns whether an entry was
    /// marked; either way the plugin then refreshes the dead players' mute state.
    pub fn apply_killed(&mut self, nickname: &str) -> bool {
        self.clients
            .get_mut(nickname)
            .map(|client| client.data_frame = INVALID_DATA_FRAME)
            .is_some()
    }
}
