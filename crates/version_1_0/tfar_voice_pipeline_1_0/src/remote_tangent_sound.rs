//! The positional remote tangent sound: a `remote_start` or `remote_end` WAV shaped for one radio
//! that hears another player's press or release.
//!
//! **Role:** the positional `PlaybackHandler::playWavFile`
//! (`reference/tfar_1_0/ts/src/PlaybackHandler.cpp:147-262`): the gain; with the local player's
//! data, the vehicle low-pass when a vehicle stands between, and on a loudspeaker the distance
//! attenuation, the speaker band-pass, the under-water low-pass and the simple ILD; a carried
//! radio's sound only keeps its ear. The radio sounds are plugin-folder files, which the
//! original places by the ILD alone (`PlaybackHandler.cpp:218`), so no HRTF stage runs.
//!
//! **Position:** called by the plugin app with a `RemoteSoundPlacement` from
//! `SessionState::apply_remote_tangent`; the finished `Playback` goes into the connection's
//! `PlaybackMixer`. Uses the filter designs of `tfar_client_state_1_0` and this crate's gains.
//!
//! **Signals and state:** none kept: each filter is made, used and dropped while the playback is
//! built. It allocates, so it runs on the thread that handles the command, never on the
//! TeamSpeak audio thread.
//!
//! **Invariants:** every stage saturates at the `i16` limits; a filter design the library refuses
//! leaves that stage out; the sound keeps its length.

use tfar_client_state_1_0::prelude::{
    CANT_SPEAK_LOW_PASS, RemoteSoundPlacement, SPEAKER_BAND_PASS, StereoMode, vehicle_low_pass,
};
use tfar_dsp_filters::{FilterDesign, StereoFilter};
use tfar_geometry::{Direction, Position};
use tfar_playback_mixer::Playback;
use tfar_radio_effects_1_0::SAMPLE_RATE_HZ;
use tfar_sample_buffer::{StereoBuffer, apply_gain};
use tfar_spatial_audio::{ILD_COEFFICIENT_1_0, apply_pan, stereo_pan_gains};

use crate::distance_gain::{
    CANT_SPEAK_GAIN, NO_VEHICLE_LOSS, SPEAKER_GAIN, gain_as_f32, vehicle_loss_gain,
    volume_attenuation,
};
use crate::error::{Error, Result};

#[cfg(test)]
#[path = "tests/remote_tangent_sound.rs"]
mod tests;

/// The gain of the under-water low-pass on a loudspeaker sound (`CANT_SPEAK_GAIN * 50`,
/// `PlaybackHandler.cpp:210`).
pub const UNDER_WATER_SPEAKER_GAIN: f32 = CANT_SPEAK_GAIN * 50.0;

/// Where the local player listens from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemoteSoundListener {
    /// The local player's position.
    pub position: Position,
    /// The local player's view direction.
    pub view: Direction,
}

/// How one remote tangent sound is shaped. The WAV itself comes separately, decoded.
#[derive(Clone, Debug, PartialEq)]
pub struct RemoteSoundPlan {
    /// Where and how loud the radio that hears the press plays.
    pub placement: RemoteSoundPlacement,
    /// The local player, when known; without, only the gain applies.
    pub listener: Option<RemoteSoundListener>,
    /// How far a loudspeaker at volume 10 carries, in metres (the game's setting).
    pub speaker_distance: f64,
}

/// The playback of `wave` (interleaved 48 kHz stereo) shaped as `plan` says. Allocates: call it
/// off the audio thread.
///
/// # Errors
///
/// [`Error::RemoteTangentSound`] when `wave` is not whole stereo frames.
pub fn render_remote_tangent_sound(wave: &[i16], plan: &RemoteSoundPlan) -> Result<Playback> {
    Playback::new(wave, |sound| {
        shape_sound(sound, &plan.placement, plan.listener, plan.speaker_distance);
    })
    .map_err(|source| Error::RemoteTangentSound { source })
}

/// Silences the ear `stereo_mode` leaves out of `buffer`.
pub fn apply_stereo_mode(buffer: &mut StereoBuffer, stereo_mode: StereoMode) {
    let silenced = match stereo_mode {
        StereoMode::Both => return,
        StereoMode::Left => 1,
        StereoMode::Right => 0,
    };
    for frame in buffer.as_mut_slice().as_chunks_mut::<2>().0 {
        if let Some(sample) = frame.get_mut(silenced) {
            *sample = 0;
        }
    }
}

/// Runs every stage of `placement` over `sound`.
fn shape_sound(
    sound: &mut StereoBuffer,
    placement: &RemoteSoundPlacement,
    listener: Option<RemoteSoundListener>,
    speaker_distance: f64,
) {
    apply_gain(sound.as_mut_slice(), gain_as_f32(placement.gain));
    let Some(listener) = listener else {
        return;
    };
    let audible = (placement.volume / 10.0 * speaker_distance).round();
    let distance = placement
        .position
        .map_or(0.0, |position| position.distance_to(listener.position));
    let loss = placement.vehicle_volume_loss;
    if loss > NO_VEHICLE_LOSS && !placement.is_same_vehicle {
        let gain =
            volume_attenuation(distance, true, audible, 1.0 - loss) * vehicle_loss_gain(loss);
        filter_sound(sound, vehicle_low_pass(loss), gain_as_f32(gain));
    }
    if !placement.is_on_ground {
        apply_stereo_mode(sound, placement.stereo_mode);
        return;
    }
    apply_gain(
        sound.as_mut_slice(),
        gain_as_f32(volume_attenuation(distance, true, audible, 1.0)),
    );
    filter_sound(sound, SPEAKER_BAND_PASS, SPEAKER_GAIN);
    if placement.is_underwater {
        filter_sound(sound, CANT_SPEAK_LOW_PASS, UNDER_WATER_SPEAKER_GAIN);
    }
    if let Some(position) = placement.position {
        let gains = stereo_pan_gains(
            listener.position.direction_to(position),
            listener.view.view_angle(),
            ILD_COEFFICIENT_1_0,
        );
        apply_pan(sound, gains);
    }
}

/// Runs `sound` through a fresh stereo filter of `design` at `gain`, or logs why it cannot be
/// built.
fn filter_sound(sound: &mut StereoBuffer, design: FilterDesign, gain: f32) {
    match StereoFilter::new(design, SAMPLE_RATE_HZ) {
        Ok(mut filter) => filter.process_stereo_i16(sound, gain),
        Err(error) => tracing::warn!(%error, "a remote sound filter could not be built"),
    }
}
