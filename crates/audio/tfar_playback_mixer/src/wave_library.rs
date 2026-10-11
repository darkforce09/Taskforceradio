//! `WaveLibrary`: the radio sound WAV files, decoded once with `hound` and cached by canonical
//! path.
//!
//! **Role:** loads a WAV file, refuses any format but 48 kHz, 2-channel, 16-bit integer PCM with an
//! error naming the file and its format, and caches the samples as a shared `Arc<[i16]>`.
//!
//! **Position:** a module of `tfar_playback_mixer`, tier 1 of crates/audio; used by the plugins to
//! preload the `radio-sounds` folder and by whoever builds a `Playback` from a sound. The
//! specification is `getWavFileFromPath` and `playbackWavStereo::construct` of
//! `reference/tfar_1_0/ts/src/PlaybackHandler.cpp`, without their leak of the decoder and its file
//! on a wrong format.
//!
//! **Signals and state:** the cache, behind a short `parking_lot` lock that is never held while a
//! file is read; callable from any thread but the audio thread, since loading reads the disk.
//!
//! **Invariants:** every cached sound is whole stereo frames of 48 kHz 16-bit samples; one file is
//! cached once, under its canonical path, however it was named.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::Mutex;

use crate::error::{Error, Result};

#[cfg(test)]
#[path = "tests/wave_library.rs"]
mod tests;

/// The only sample rate the mixer plays, TeamSpeak's.
const REQUIRED_SAMPLE_RATE: u32 = 48_000;
/// The only channel count the mixer plays.
const REQUIRED_CHANNELS: u16 = 2;
/// The only sample width the mixer plays.
const REQUIRED_BITS_PER_SAMPLE: u16 = 16;

/// Decoded radio sounds by canonical path.
#[derive(Debug, Default)]
pub struct WaveLibrary {
    sounds: Mutex<HashMap<PathBuf, Arc<[i16]>>>,
}

impl WaveLibrary {
    /// An empty library.
    pub fn new() -> Self {
        Self::default()
    }

    /// The interleaved stereo samples of the WAV file at `path`, from the cache or decoded now.
    ///
    /// # Errors
    ///
    /// [`Error::WavePath`] when the path cannot be resolved, [`Error::WaveRead`] when `hound`
    /// cannot read the file, [`Error::UnsupportedWaveFormat`] for anything but 48 kHz, 2-channel,
    /// 16-bit integer PCM, and [`Error::PartialWaveFrame`] when the file ends inside a frame.
    pub fn load(&self, path: impl AsRef<Path>) -> Result<Arc<[i16]>> {
        let canonical = canonical_path(path.as_ref())?;
        if let Some(cached) = self.sounds.lock().get(&canonical) {
            return Ok(Arc::clone(cached));
        }
        let decoded = decode(&canonical)?;
        let mut sounds = self.sounds.lock();
        Ok(Arc::clone(sounds.entry(canonical).or_insert(decoded)))
    }

    /// Loads every file of `paths` into the cache; returns one error, naming its file, for each
    /// file that could not be loaded. The other files are cached either way.
    pub fn preload<P: AsRef<Path>>(&self, paths: impl IntoIterator<Item = P>) -> Vec<Error> {
        paths
            .into_iter()
            .filter_map(|path| self.load(path).err())
            .collect()
    }

    /// The cached samples of `path`, without reading the file when it is not cached.
    pub fn cached(&self, path: impl AsRef<Path>) -> Option<Arc<[i16]>> {
        let canonical = canonical_path(path.as_ref()).ok()?;
        self.sounds.lock().get(&canonical).map(Arc::clone)
    }

    /// The number of sounds in the cache.
    pub fn len(&self) -> usize {
        self.sounds.lock().len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.sounds.lock().is_empty()
    }
}

/// `path` resolved to its canonical form, the cache key.
fn canonical_path(path: &Path) -> Result<PathBuf> {
    std::fs::canonicalize(path).map_err(|source| Error::WavePath {
        path: path.to_path_buf(),
        source,
    })
}

/// Decodes the WAV file at `path`, refusing every format but 48 kHz, 2-channel, 16-bit integer.
fn decode(path: &Path) -> Result<Arc<[i16]>> {
    let read_error = |source| Error::WaveRead {
        path: path.to_path_buf(),
        source,
    };
    let reader = hound::WavReader::open(path).map_err(read_error)?;
    let spec = reader.spec();
    if spec.sample_rate != REQUIRED_SAMPLE_RATE
        || spec.channels != REQUIRED_CHANNELS
        || spec.bits_per_sample != REQUIRED_BITS_PER_SAMPLE
        || spec.sample_format != hound::SampleFormat::Int
    {
        return Err(Error::UnsupportedWaveFormat {
            path: path.to_path_buf(),
            sample_rate: spec.sample_rate,
            channels: spec.channels,
            bits_per_sample: spec.bits_per_sample,
            sample_format: spec.sample_format,
        });
    }
    let samples = reader
        .into_samples::<i16>()
        .collect::<core::result::Result<Vec<i16>, hound::Error>>()
        .map_err(read_error)?;
    if !samples.len().is_multiple_of(usize::from(REQUIRED_CHANNELS)) {
        return Err(Error::PartialWaveFrame {
            path: path.to_path_buf(),
            samples: samples.len(),
        });
    }
    Ok(Arc::from(samples))
}
