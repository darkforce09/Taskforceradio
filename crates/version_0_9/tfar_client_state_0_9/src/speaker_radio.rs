//! `SpeakerRadio`: one radio playing on loudspeaker, filed under one of its frequencies.

use std::sync::Arc;

use tfar_game_commands_0_9::{SpeakerRecord, VehicleDescriptor};
use tfar_geometry::Position;

#[cfg(test)]
#[path = "tests/speaker_radio.rs"]
mod tests;

/// The wave height below which a loudspeaker radio sounds under water (`UNDERWATER_LEVEL`,
/// `reference/tfar_0_9_12/ts/src/plugin.cpp:60`).
pub const UNDER_WATER_WAVE_Z: f64 = -1.1;

/// One radio on loudspeaker near the local player (`SPEAKER_DATA`,
/// `reference/tfar_0_9_12/ts/src/plugin.cpp:375`); the server state files one copy under each
/// frequency the radio listens on.
#[derive(Clone, Debug, PartialEq)]
pub struct SpeakerRadio {
    /// The radio's id (a network id or a radio class name), which keys its sound state; shared,
    /// so a reception names it without copying the text.
    pub radio_id: Arc<str>,
    /// The nickname of the player who carries the radio.
    pub owner_nickname: String,
    /// The radio's position relative to the local listener; `None` when its owner carries it,
    /// and the owner's position stands in.
    pub position: Option<Position>,
    /// The radio's volume.
    pub volume: i32,
    /// The vehicle the radio is in; shared, so a reception names it without copying.
    pub vehicle: Arc<VehicleDescriptor>,
    /// The radio's height above sea level.
    pub wave_z: f64,
}

impl SpeakerRadio {
    /// The radio `record` describes, without its frequency list.
    pub fn from_record(record: &SpeakerRecord) -> Self {
        Self {
            radio_id: Arc::from(record.radio_id.as_str()),
            owner_nickname: record.owner_nickname.clone(),
            position: record.position,
            volume: record.volume,
            vehicle: Arc::new(record.vehicle.clone()),
            wave_z: record.wave_z,
        }
    }

    /// Whether the radio sounds under water: its wave height is below [`UNDER_WATER_WAVE_Z`].
    pub fn is_under_water(&self) -> bool {
        is_under_water(self.wave_z)
    }
}

/// Whether a wave height `wave_z` lies under water, below [`UNDER_WATER_WAVE_Z`].
pub fn is_under_water(wave_z: f64) -> bool {
    wave_z < UNDER_WATER_WAVE_Z
}
