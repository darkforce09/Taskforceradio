//! `HrirLibrary`: the HRIR sphere, loaded once and resampled to 48 kHz, ready for every source.
//!
//! **Role:** reads the sphere file, refuses a malformed one, lets the `hrtf` crate resample it to
//! [`SAMPLE_RATE_HZ`] and transform it into the frequency domain for the block shape of
//! `render_timing`, and keeps that prototype processor for [`crate::hrtf::HrtfSource`] to copy.
//!
//! **Position:** a module of `tfar_audio::hrtf`; built by the plugin at start (or when the HRTF is turned
//! on), shared as `Arc<HrirLibrary>` with the voice pipelines, which create one
//! [`crate::hrtf::HrtfSource`] per sound source from it.
//!
//! **Signals and state:** immutable after `load`; `Send + Sync`, read from any thread. `load` is
//! heavy (resampling 2 × 187 impulse responses and one FFT per ear and point) and runs off the
//! audio thread.
//!
//! **Invariants:** a library exists only for a sphere that passed the structural check and the
//! `hrtf` crate's parser; its prototype processor renders blocks of
//! `INTERPOLATION_STEPS × CONVOLUTION_BLOCK_FRAMES` frames.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use hrtf::{HrirSphere, HrtfProcessor};

use crate::hrtf::error::{Error, Result};
use crate::hrtf::render_timing::{CONVOLUTION_BLOCK_FRAMES, INTERPOLATION_STEPS, SAMPLE_RATE_HZ};
use crate::hrtf::sphere_file_check::check_sphere_file;

#[cfg(test)]
#[path = "tests/hrir_library.rs"]
mod tests;

/// The HRIR sphere loaded from one file, resampled to 48 kHz and prepared for rendering.
///
/// Load it once with [`HrirLibrary::load`] and share the `Arc`; every [`crate::hrtf::HrtfSource`] made
/// from it copies the prepared sphere, so sources never share mutable state.
pub struct HrirLibrary {
    prototype: HrtfProcessor,
    hrir_length: usize,
    point_count: usize,
    recorded_sample_rate_hz: u32,
    source_path: PathBuf,
}

impl HrirLibrary {
    /// Loads the sphere at `path`, checks it, resamples it to 48 kHz and prepares it for
    /// rendering. Heavy (tens of milliseconds and a few megabytes); call it off the audio thread.
    ///
    /// # Errors
    ///
    /// [`Error::SphereFileUnreadable`] when the file cannot be read, [`Error::SphereFileMalformed`]
    /// when it is not a well-formed sphere, and [`Error::SphereRejected`] when the `hrtf` crate
    /// refuses it.
    pub fn load(path: impl AsRef<Path>) -> Result<Arc<Self>> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| Error::SphereFileUnreadable {
            path: path.to_owned(),
            source,
        })?;
        let layout = check_sphere_file(&bytes).map_err(|reason| Error::SphereFileMalformed {
            path: path.to_owned(),
            reason,
        })?;
        let sphere = HrirSphere::new(bytes.as_slice(), SAMPLE_RATE_HZ).map_err(|error| {
            Error::SphereRejected {
                path: path.to_owned(),
                detail: format!("{error:?}"),
            }
        })?;
        let hrir_length = sphere.len();
        let prototype = HrtfProcessor::new(sphere, INTERPOLATION_STEPS, CONVOLUTION_BLOCK_FRAMES);
        Ok(Arc::new(Self {
            prototype,
            hrir_length,
            point_count: layout.point_count,
            recorded_sample_rate_hz: layout.sample_rate_hz,
            source_path: path.to_owned(),
        }))
    }

    /// The samples in each impulse response per ear after resampling to 48 kHz.
    pub fn hrir_length(&self) -> usize {
        self.hrir_length
    }

    /// The measured directions in the sphere.
    pub fn point_count(&self) -> usize {
        self.point_count
    }

    /// The rate the sphere was recorded at, before resampling.
    pub fn recorded_sample_rate_hz(&self) -> u32 {
        self.recorded_sample_rate_hz
    }

    /// The file the sphere was loaded from.
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    /// A fresh processor for one source: a copy of the prepared sphere and its scratch buffers.
    /// Allocates the size of the prepared sphere.
    pub(crate) fn new_processor(&self) -> HrtfProcessor {
        self.prototype.clone()
    }
}

impl core::fmt::Debug for HrirLibrary {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("HrirLibrary")
            .field("source_path", &self.source_path)
            .field("point_count", &self.point_count)
            .field("hrir_length", &self.hrir_length)
            .field("recorded_sample_rate_hz", &self.recorded_sample_rate_hz)
            .finish_non_exhaustive()
    }
}
