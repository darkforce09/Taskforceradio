//! The positional remote push-to-talk sound: a `remote_start` or `remote_end` WAV placed where
//! the listener hears the transmission, on their own radio or on a loudspeaker radio.
//!
//! **Role:** the positional `playWavFile` (`reference/tfar_0_9_12/ts/src/plugin.cpp:711-785`):
//! the decoded WAV at the receiving radio's volume and the global volume; with current data for
//! the listener, the vehicle low-pass when a vehicle stands between, and on a loudspeaker the
//! distance gain, the speaker band-pass and the under-water low-pass; last the direction, by the
//! HRIR sphere or, without one, the 0.9 simple ILD.
//!
//! **Position:** called by the plugin app when an incoming push-to-talk asks for a sound; the
//! finished `Playback` goes into the connection's `PlaybackMixer`, which the mixed-playback
//! callback plays. Uses the gains and filter designs the voice chain uses.
//!
//! **Signals and state:** none kept: the filters and the HRTF source of one sound are made,
//! used and dropped while the playback is built. It allocates, so it runs on the thread that
//! handles the command, never on the TeamSpeak audio thread.
//!
//! **Invariants:** every stage saturates at the `i16` limits; the HRTF's constant latency is cut
//! off, so the sound starts at once; a filter design the library refuses leaves that stage out.

use tfar_client_state_0_9::prelude::{Angle, Position};
use tfar_client_state_0_9::{
    CANT_SPEAK_LOW_PASS, FILTER_SAMPLE_RATE_HZ, SPEAKER_BAND_PASS, vehicle_low_pass,
};
use tfar_dsp_filters::{FilterDesign, StereoFilter};
use tfar_hrtf::{ADDED_LATENCY_FRAMES, HrirLibrary, HrtfSource, ListenerRelativeDirection};
use tfar_playback_mixer::Playback;
use tfar_sample_buffer::{StereoBuffer, apply_gain};
use tfar_spatial_audio::{ILD_COEFFICIENT_0_9, apply_pan, stereo_pan_gains};

use crate::distance_gain::volume_from_distance;
use crate::error::{Error, Result};
use crate::voice_gains::{
    CANT_SPEAK_GAIN, SPEAKER_GAIN, as_stage_gain, hrtf_makeup_gain, vehicle_loss_gain,
};

#[cfg(test)]
#[path = "tests/remote_tangent_sound.rs"]
mod tests;

/// The extra gain of the under-water low-pass on a loudspeaker sound: the original runs it at
/// `CANT_SPEAK_GAIN * 50` (plugin.cpp:762), not at the voice path's ×14.
pub const UNDER_WATER_SOUND_GAIN_FACTOR: f64 = 50.0;

/// Where the local player listens from, for a sound with current listener data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ListenerView {
    /// The listener's position (the origin: the game sends positions relative to the listener).
    pub position: Position,
    /// The direction the listener looks.
    pub view_angle: Angle,
    /// The combined isolation of the vehicles between listener and radio, when it muffles.
    pub vehicle_isolation: Option<f64>,
}

/// How one remote push-to-talk sound is shaped, as the incoming command planned it. The WAV
/// itself comes separately, decoded.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemoteSoundPlan {
    /// The receiving radio's volume gain times the global volume.
    pub gain: f64,
    /// Where the receiving radio is, relative to the listener.
    pub position: Position,
    /// Whether the radio plays on a loudspeaker.
    pub is_on_loudspeaker: bool,
    /// The receiving radio's volume, which sets a loudspeaker's reach.
    pub radio_volume: i32,
    /// Whether the receiving radio is under water.
    pub is_under_water: bool,
    /// The reach of a loudspeaker at volume 10, in metres.
    pub speaker_distance: f64,
    /// The listener, when they have current data; without, only the gain applies.
    pub listener: Option<ListenerView>,
}

/// The playback of `wave` (interleaved 48 kHz stereo) shaped as `plan` says, with direction from
/// `hrir` when given and the simple ILD otherwise. Allocates: call it off the audio thread.
///
/// # Errors
///
/// [`Error::RemoteTangentSound`] when `wave` is not whole stereo frames.
pub fn render_remote_tangent_sound(
    wave: &[i16],
    plan: &RemoteSoundPlan,
    hrir: Option<&HrirLibrary>,
) -> Result<Playback> {
    Playback::new(wave, |sound| shape_sound(sound, plan, hrir))
        .map_err(|source| Error::RemoteTangentSound { source })
}

/// Runs every stage of `plan` over `sound`.
fn shape_sound(sound: &mut StereoBuffer, plan: &RemoteSoundPlan, hrir: Option<&HrirLibrary>) {
    apply_gain(sound.as_mut_slice(), as_stage_gain(plan.gain));
    let Some(listener) = plan.listener else {
        return;
    };
    let reach = whole_metres(f64::from(plan.radio_volume) / 10.0 * plan.speaker_distance);
    let distance = listener.position.distance_to(plan.position);
    if let Some(isolation) = listener.vehicle_isolation {
        let gain = volume_from_distance(distance, true, reach, 1.0 - isolation)
            * vehicle_loss_gain(isolation);
        filter_sound(sound, vehicle_low_pass(isolation), gain);
    }
    if plan.is_on_loudspeaker {
        apply_gain(
            sound.as_mut_slice(),
            as_stage_gain(volume_from_distance(distance, true, reach, 1.0)),
        );
        filter_sound(sound, SPEAKER_BAND_PASS, SPEAKER_GAIN);
        if plan.is_under_water {
            filter_sound(
                sound,
                CANT_SPEAK_LOW_PASS,
                CANT_SPEAK_GAIN * UNDER_WATER_SOUND_GAIN_FACTOR,
            );
        }
    }
    place_sound(sound, &listener, plan.position, hrir);
}

/// Runs `sound` through a fresh stereo filter of `design` at `gain`.
fn filter_sound(sound: &mut StereoBuffer, design: FilterDesign, gain: f64) {
    match StereoFilter::new(design, FILTER_SAMPLE_RATE_HZ) {
        Ok(mut filter) => filter.process_stereo_i16(sound, as_stage_gain(gain)),
        Err(error) => tracing::warn!(%error, "a remote sound filter could not be designed"),
    }
}

/// Places `sound` from `listener` towards `source`: binaurally through a one-shot HRTF source
/// at the HRTF makeup gain, its latency cut off, or by the simple ILD without a sphere.
fn place_sound(
    sound: &mut StereoBuffer,
    listener: &ListenerView,
    source: Position,
    hrir: Option<&HrirLibrary>,
) {
    let Some(library) = hrir else {
        let gains = stereo_pan_gains(
            listener.position.direction_to(source),
            listener.view_angle,
            ILD_COEFFICIENT_0_9,
        );
        apply_pan(sound, gains);
        return;
    };
    let frames = sound.frames();
    let mut padded = StereoBuffer::with_frames(frames.saturating_add(ADDED_LATENCY_FRAMES));
    for (target, sample) in padded.as_mut_slice().iter_mut().zip(sound.as_slice()) {
        *target = *sample;
    }
    let mut rendered = StereoBuffer::with_frames(0);
    let direction =
        ListenerRelativeDirection::from_arma(listener.position, listener.view_angle, source);
    HrtfSource::new(library).process(
        &padded,
        &mut rendered,
        direction,
        as_stage_gain(hrtf_makeup_gain()),
    );
    let latency_samples = ADDED_LATENCY_FRAMES.saturating_mul(StereoBuffer::CHANNELS);
    for (target, sample) in sound
        .as_mut_slice()
        .iter_mut()
        .zip(rendered.as_slice().iter().skip(latency_samples))
    {
        *target = *sample;
    }
}

/// `metres` truncated to whole metres, as the original passes a float to its `int` parameter.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the original's float-to-int conversion truncates; a reach beyond i32 saturates"
)]
fn whole_metres(metres: f64) -> i32 {
    metres.trunc() as i32
}
