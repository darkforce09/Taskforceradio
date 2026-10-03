//! The radio sounds: the WAV files under the plugins folder, the HRIR sphere, the playback mixer,
//! and the shaping of the local and remote tangent sounds before they are queued.
//!
//! **Role:** `PlaybackHandler` of `reference/tfar_1_0/ts/src/PlaybackHandler.cpp:133-262`: load
//! `radio-sounds/` once at init, play `on` and `off` through TeamSpeak, and turn a local or remote
//! tangent sound into a mixer playback with its gain, ear, vehicle and loudspeaker filters,
//! under-water muffling and simple ILD direction.
//!
//! **Position:** owned by `PluginRuntime`; [`RadioSounds::play_local`] and
//! [`RadioSounds::play_remote`] run on the game link worker and TeamSpeak's main thread, the
//! mixer's `mix_into` on the mixed-playback audio callback.
//!
//! **Signals and state:** the decoded WAV cache and the mixer, both safe to share; the shaping
//! runs on the calling thread, never on an audio thread.
//!
//! **Invariants:** a missing or malformed WAV plays nothing and is logged once at init; a missing
//! HRIR sphere switches HRTF off for the session.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tfar_arma_values::{RadioSubtype, StereoMode};
use tfar_client_state_1_0::prelude::RemoteSoundPlacement;
use tfar_dsp_filters::prelude::{FilterDesign, StereoFilter};
use tfar_geometry::prelude::{Direction, Position};
use tfar_hrtf::HrirLibrary;
use tfar_playback_mixer::{Playback, PlaybackMixer, WaveLibrary};
use tfar_sample_buffer::{StereoBuffer, apply_gain};
use tfar_spatial_audio::prelude::{ILD_COEFFICIENT_1_0, apply_pan, stereo_pan_gains};
use tfar_voice_pipeline_1_0::prelude::{
    CANT_SPEAK_GAIN, NO_VEHICLE_LOSS, SPEAKER_GAIN, vehicle_loss_gain, volume_attenuation,
};

use crate::runtime::game_lifecycle::wave_path;

#[cfg(test)]
#[path = "tests/radio_sounds.rs"]
mod tests;

/// The HRIR sphere, relative to the plugins folder (the `cargo xtask dist` layout).
pub const HRIR_SPHERE: &str = "tfar_hrir/IRC_1002_C.bin";

/// Every sound the plugin plays, relative to the plugins folder and without `.wav`.
pub const RADIO_SOUNDS: [&str; 14] = [
    "radio-sounds/on",
    "radio-sounds/off",
    "radio-sounds/sw/local_start",
    "radio-sounds/sw/local_end",
    "radio-sounds/sw/remote_start",
    "radio-sounds/sw/remote_end",
    "radio-sounds/lr/local_start",
    "radio-sounds/lr/local_end",
    "radio-sounds/lr/remote_start",
    "radio-sounds/lr/remote_end",
    "radio-sounds/ab/local_start",
    "radio-sounds/ab/local_end",
    "radio-sounds/ab/remote_start",
    "radio-sounds/ab/remote_end",
];

/// The sample rate of every radio sound.
const SAMPLE_RATE_HZ: f64 = 48_000.0;

/// The under-water loudspeaker gain (`CANT_SPEAK_GAIN * 50`).
const UNDERWATER_SPEAKER_GAIN: f32 = CANT_SPEAK_GAIN * 50.0;

/// The local tangent sound of `subtype` (`SoundDirectory::getRadioSound`,
/// `PlaybackHandler.hpp:290-303`): only `digital`, `digital_lr` and `airborne` radios have one.
pub fn local_radio_sound(subtype: &RadioSubtype, is_pressed: bool) -> Option<String> {
    let folder = match subtype {
        RadioSubtype::Digital | RadioSubtype::DigitalLongRange | RadioSubtype::Airborne => {
            subtype.sound_folder()?
        }
        _ => return None,
    };
    let event = if is_pressed {
        "local_start"
    } else {
        "local_end"
    };
    Some(format!("radio-sounds/{folder}/{event}"))
}

/// Silences the ear `stereo_mode` leaves out.
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

/// Runs `design` over `buffer` at `gain`, or logs why it cannot be built.
fn filter(buffer: &mut StereoBuffer, design: FilterDesign, gain: f32) {
    match StereoFilter::new(design, SAMPLE_RATE_HZ) {
        Ok(mut filter) => filter.process_stereo_i16(buffer, gain),
        Err(error) => tracing::warn!(%error, "a radio sound filter could not be built"),
    }
}

/// Narrows a gain to the `f32` the sample crates take.
#[expect(
    clippy::cast_possible_truncation,
    reason = "gains are small finite values; the original computes them in f32"
)]
const fn as_gain(value: f64) -> f32 {
    value as f32
}

/// Shapes a remote tangent sound as `PlaybackHandler::playWavFile` with a position does: the
/// gain, the vehicle low-pass, the loudspeaker attenuation and band-pass, the under-water
/// low-pass and the simple ILD from the listener; a carried radio's sound only keeps its ear.
/// `listener` is the local player's position and view, `speaker_distance` the game's setting.
pub fn shape_remote_sound(
    buffer: &mut StereoBuffer,
    placement: &RemoteSoundPlacement,
    listener: Option<(Position, Direction)>,
    speaker_distance: f64,
) {
    apply_gain(buffer.as_mut_slice(), as_gain(placement.gain));
    let Some((listener_position, view)) = listener else {
        return;
    };
    let audible = (placement.volume / 10.0 * speaker_distance).round();
    let distance = placement
        .position
        .map_or(0.0, |position| position.distance_to(listener_position));
    let loss = placement.vehicle_volume_loss;
    if loss > NO_VEHICLE_LOSS && !placement.is_same_vehicle {
        let gain =
            volume_attenuation(distance, true, audible, 1.0 - loss) * vehicle_loss_gain(loss);
        let cutoff_hz = 20_000.0 * (1.0 - loss) / 4.0;
        filter(
            buffer,
            FilterDesign::ButterworthLowPass {
                order: 2,
                cutoff_hz,
            },
            as_gain(gain),
        );
    }
    if placement.is_on_ground {
        apply_gain(
            buffer.as_mut_slice(),
            as_gain(volume_attenuation(distance, true, audible, 1.0)),
        );
        filter(
            buffer,
            FilterDesign::ButterworthBandPass {
                order: 1,
                centre_hz: 2_000.0,
                width_hz: 1_000.0,
            },
            SPEAKER_GAIN,
        );
        if placement.is_underwater {
            filter(
                buffer,
                FilterDesign::ButterworthLowPass {
                    order: 4,
                    cutoff_hz: 100.0,
                },
                UNDERWATER_SPEAKER_GAIN,
            );
        }
        if let Some(position) = placement.position {
            let gains = stereo_pan_gains(
                listener_position.direction_to(position),
                view.view_angle(),
                ILD_COEFFICIENT_1_0,
            );
            apply_pan(buffer, gains);
        }
    } else {
        apply_stereo_mode(buffer, placement.stereo_mode);
    }
}

/// The radio sounds, the mixer and the HRIR sphere; see the module header.
#[derive(Debug)]
pub struct RadioSounds {
    /// TeamSpeak's plugins folder; `None` when TeamSpeak did not say.
    plugin_path: Option<PathBuf>,
    /// The decoded WAV files.
    waves: WaveLibrary,
    /// The playbacks and the own-voice stream the mixed-playback callback adds in.
    pub mixer: PlaybackMixer,
    /// The HRIR sphere; `None` switches HRTF off.
    hrir: Option<Arc<HrirLibrary>>,
}

impl RadioSounds {
    /// Loads every radio sound and the HRIR sphere under `plugin_path`; returns the sounds and
    /// the problems to tell the user (a missing sphere).
    pub fn load(plugin_path: Option<PathBuf>) -> (Self, Vec<String>) {
        let waves = WaveLibrary::new();
        let mut problems = Vec::new();
        let mut hrir = None;
        if let Some(folder) = plugin_path.as_deref() {
            for error in waves.preload(RADIO_SOUNDS.iter().map(|sound| wave_path(folder, sound))) {
                tracing::warn!(%error, "a radio sound could not be loaded");
            }
            let sphere = folder.join(HRIR_SPHERE);
            match HrirLibrary::load(&sphere) {
                Ok(library) => hrir = Some(library),
                Err(error) => {
                    tracing::warn!(%error, "the HRIR sphere could not be loaded; HRTF is off");
                    problems.push(format!(
                        "Task Force Radio: the HRIR sphere {} is missing or unreadable ({error}); \
                         voices are placed with the simple stereo panning instead.",
                        sphere.display()
                    ));
                }
            }
        }
        let sounds = Self {
            plugin_path,
            waves,
            mixer: PlaybackMixer::new(),
            hrir,
        };
        (sounds, problems)
    }

    /// The HRIR sphere, while HRTF is on.
    pub fn hrir_library(&self) -> Option<&HrirLibrary> {
        self.hrir.as_deref()
    }

    /// The file of `sound` (relative, without `.wav`).
    pub fn sound_path(&self, sound: &str) -> Option<PathBuf> {
        self.plugin_path
            .as_deref()
            .map(|folder: &Path| wave_path(folder, sound))
    }

    /// The decoded samples of `sound`.
    fn samples(&self, sound: &str) -> Option<Arc<[i16]>> {
        let path = self.sound_path(sound)?;
        self.waves
            .load(&path)
            .inspect_err(|error| tracing::debug!(%error, "a radio sound is unavailable"))
            .ok()
    }

    /// Queues `sound` at `gain` into the ear `stereo_mode` names (the local tangent sounds).
    pub fn play_local(&self, sound: &str, gain: f64, stereo_mode: StereoMode) {
        let Some(samples) = self.samples(sound) else {
            return;
        };
        let playback = Playback::new(&samples, |buffer| {
            apply_gain(buffer.as_mut_slice(), as_gain(gain));
            apply_stereo_mode(buffer, stereo_mode);
        });
        self.queue(playback);
    }

    /// Queues `sound` shaped for one radio that hears a remote tangent.
    pub fn play_remote(
        &self,
        sound: &str,
        placement: &RemoteSoundPlacement,
        listener: Option<(Position, Direction)>,
        speaker_distance: f64,
    ) {
        let Some(samples) = self.samples(sound) else {
            return;
        };
        let playback = Playback::new(&samples, |buffer| {
            shape_remote_sound(buffer, placement, listener, speaker_distance);
        });
        self.queue(playback);
    }

    /// Hands a built playback to the mixer.
    fn queue(&self, playback: tfar_playback_mixer::Result<Playback>) {
        match playback.and_then(|playback| self.mixer.add(playback)) {
            Ok(_) => {}
            Err(error) => tracing::warn!(%error, "a radio sound could not be queued"),
        }
    }

    /// Drops the playbacks that finished, off the audio thread.
    pub fn release_finished(&self) -> usize {
        self.mixer.release_finished()
    }
}
