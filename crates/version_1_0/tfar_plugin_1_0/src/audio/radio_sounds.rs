//! The radio sounds: the WAV files under the plugins folder, the HRIR sphere, the playback mixer
//! with the own-voice stream, and the local and remote tangent sounds queued into it.
//!
//! **Role:** `PlaybackHandler` of `reference/tfar_1_0/ts/src/PlaybackHandler.cpp:133-262`: load
//! `radio-sounds/` once at init, play `on` and `off` through TeamSpeak, queue a local tangent
//! sound at its gain into its ear, and queue a remote tangent sound shaped by
//! `tfar_plugin_logic_1_0::voice_pipeline::render_remote_tangent_sound`.
//!
//! **Position:** owned by `PluginRuntime`; [`RadioSounds::play_local`] and
//! [`RadioSounds::play_remote`] run on the game link worker and TeamSpeak's main thread,
//! [`RadioSounds::push_own_voice`] on the captured-voice audio callback, the mixer's `mix_into`
//! on the mixed-playback audio callback.
//!
//! **Signals and state:** the decoded WAV cache and the mixer, both safe to share; the own-voice
//! `StreamProducer` behind a mutex only the captured-voice callback takes, with `try_lock`; the
//! shaping runs on the calling thread, never on an audio thread.
//!
//! **Invariants:** a missing or malformed WAV plays nothing and is logged once at init; a missing
//! HRIR sphere switches HRTF off for the session; pushing the own voice never blocks or
//! allocates.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::Mutex;
use tfar_audio::hrtf::HrirLibrary;
use tfar_audio::playback_mixer::{Playback, PlaybackMixer, StreamProducer, WaveLibrary};
use tfar_audio::sample_buffer::apply_gain;
use tfar_foundation::arma_values::StereoMode;
use tfar_plugin_logic_1_0::voice_pipeline::prelude::{
    OWN_RADIO_VOICE_STREAM, RemoteSoundPlan, apply_stereo_mode, render_remote_tangent_sound,
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

/// Narrows a gain to the `f32` the sample crates take.
#[expect(
    clippy::cast_possible_truncation,
    reason = "gains are small finite values; the original computes them in f32"
)]
const fn as_gain(value: f64) -> f32 {
    value as f32
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
    /// The writing end of the mixer's [`OWN_RADIO_VOICE_STREAM`]; `None` when the mixer refused
    /// it, which leaves the speaker radios without the own voice.
    own_voice: Mutex<Option<StreamProducer>>,
    /// The HRIR sphere; `None` switches HRTF off.
    hrir: Option<Arc<HrirLibrary>>,
}

impl RadioSounds {
    /// Loads every radio sound and the HRIR sphere under `plugin_path` and opens the own-voice
    /// stream; returns the sounds and the problems to tell the user (a missing sphere).
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
        let mixer = PlaybackMixer::new();
        let own_voice = mixer
            .stream_producer(OWN_RADIO_VOICE_STREAM)
            .inspect_err(|error| tracing::warn!(%error, "the own-voice stream did not open"))
            .ok();
        let sounds = Self {
            plugin_path,
            waves,
            mixer,
            own_voice: Mutex::new(own_voice),
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
        if let Err(error) = playback.and_then(|playback| self.mixer.add(playback)) {
            tracing::warn!(%error, "a local radio sound could not be queued");
        }
    }

    /// Queues `sound` shaped as `plan` says for one radio that hears a remote tangent.
    pub fn play_remote(&self, sound: &str, plan: &RemoteSoundPlan) {
        let Some(samples) = self.samples(sound) else {
            return;
        };
        let playback = match render_remote_tangent_sound(&samples, plan) {
            Ok(playback) => playback,
            Err(error) => {
                tracing::warn!(%error, "a remote radio sound could not be shaped");
                return;
            }
        };
        if let Err(error) = self.mixer.add(playback) {
            tracing::warn!(%error, "a remote radio sound could not be queued");
        }
    }

    /// Appends the interleaved stereo `samples` to the own-voice stream; false when they were
    /// not appended (no stream, or another push at the same moment). Never blocks and never
    /// allocates: the captured-voice callback calls it.
    pub fn push_own_voice(&self, samples: &[i16]) -> bool {
        let Some(mut own_voice) = self.own_voice.try_lock() else {
            return false;
        };
        own_voice
            .as_mut()
            .is_some_and(|producer| producer.push(samples) > 0)
    }

    /// Drops the playbacks that finished, off the audio thread.
    pub fn release_finished(&self) -> usize {
        self.mixer.release_finished()
    }
}
