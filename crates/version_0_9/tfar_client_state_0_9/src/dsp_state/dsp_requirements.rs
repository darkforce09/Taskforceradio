//! `DspRequirements`: which sound state a sender's voice needs on the next frames.

use tfar_game_commands_0_9::prelude::RadioSubtype;

/// The sound state a sender's voice passes through, as the routing decides it: the direct voice
/// when the listener is in voice range, and one entry per radio the listener hears the sender
/// over. The plugin computes it while handling a command and hands it to
/// [`crate::ClientData::ensure_dsp`], so the audio thread finds every object it needs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DspRequirements {
    /// The direct voice, when the listener is within the sender's voice range.
    pub direct_voice: Option<DirectVoiceRequirement>,
    /// The radios the listener hears the sender over, in the routing's order.
    pub radios: Vec<RadioRequirement>,
}

/// The direct voice's needs: the direction source always, plus the vehicle low-pass or the
/// can't-speak low-pass of the branch the voice takes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectVoiceRequirement {
    /// The combined vehicle isolation when the voice is muffled by a vehicle, else `None`.
    pub vehicle_isolation: Option<f64>,
    /// Whether the voice takes the can't-speak branch (either side cannot speak).
    pub needs_cant_speak_filter: bool,
}

/// One radio the listener hears the sender over.
#[derive(Clone, Debug, PartialEq)]
pub struct RadioRequirement {
    /// The radio id that keys the state: `local_radio` for the listener's own radios, the
    /// loudspeaker radio's id otherwise.
    pub radio_id: String,
    /// The effect the sender's radio subtype selects.
    pub effect: RadioEffectKind,
    /// The loudspeaker stages, when the radio plays on loudspeaker.
    pub speaker: Option<SpeakerRequirement>,
}

/// The stages a loudspeaker radio adds after the radio effect.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpeakerRequirement {
    /// The combined vehicle isolation between the listener and the radio when it is muffled.
    pub vehicle_isolation: Option<f64>,
    /// Whether the radio sounds under water (the can't-speak low-pass).
    pub is_under_water: bool,
}

/// The radio effect a sender's subtype selects in the stereo function
/// (`reference/tfar_0_9_12/ts/src/plugin.cpp:2735-2758`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RadioEffectKind {
    /// `digital`: the personal (short-range) radio effect.
    Personal,
    /// `digital_lr` and `airborne`: the long-range radio effect.
    LongRange,
    /// `dd`: the underwater radio effect.
    UnderWater,
    /// `phone`: the phone band-pass.
    Phone,
    /// Any other subtype: the radio stays silent.
    Silent,
}

impl RadioEffectKind {
    /// The effect `subtype` selects.
    pub fn for_subtype(subtype: &RadioSubtype) -> Self {
        match subtype {
            RadioSubtype::Digital => Self::Personal,
            RadioSubtype::DigitalLongRange | RadioSubtype::Airborne => Self::LongRange,
            RadioSubtype::DiverDigital => Self::UnderWater,
            RadioSubtype::Phone => Self::Phone,
            RadioSubtype::Unknown(_) => Self::Silent,
        }
    }
}
