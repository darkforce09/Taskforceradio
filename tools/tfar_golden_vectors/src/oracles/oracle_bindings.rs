//! The FFI boundary to the C++ shim: the shared `#[repr(C)]` structs, the `extern "C"`
//! declarations and the one safe call per shim function.
//!
//! **Role:** the only module of the crate that holds `unsafe`: it turns each shim function of
//! `shim/oracle_shim.cpp` into a safe function over slices and references, checking every buffer
//! length before a pointer crosses the boundary.
//!
//! **Position:** inside `oracles`; called by the filter, compressor and spatial oracles, which
//! validate parameters and shape their buffers first. The shim calls DSPFilters, simpleSource and
//! F3DAudio, built by the crate's build script.
//!
//! **Signals and state:** none on the Rust side. Each shim call builds its filter, compressor or
//! F3DAudio handle on the stack and drops it before returning; the shim's failed-check counter is
//! thread-local, so calls from parallel test threads do not mix. Any thread may call.
//!
//! **Invariants:** every pointer handed to the shim comes from a live slice or reference, with the
//! element count taken from that slice; the shim catches every C++ exception and returns a status,
//! so nothing unwinds into Rust; a non-zero status becomes [`Error::OracleRefused`].

use crate::error::{Error, Result};

#[cfg(test)]
#[path = "tests/oracle_bindings.rs"]
mod tests;

/// The most distance curve points a [`ShimSpatialScene`] carries, `MAX_CURVE_POINTS` in the shim.
pub(crate) const SHIM_MAX_CURVE_POINTS: usize = 32;

/// The shim's filter kind of a DSPFilters `Butterworth::LowPass`.
pub(crate) const KIND_BUTTERWORTH_LOW_PASS: i32 = 0;
/// The shim's filter kind of a DSPFilters `Butterworth::BandPass`.
pub(crate) const KIND_BUTTERWORTH_BAND_PASS: i32 = 1;
/// The shim's filter kind of a DSPFilters `RBJ::LowPass`.
pub(crate) const KIND_RBJ_LOW_PASS: i32 = 2;
/// The shim's filter kind of a DSPFilters `RBJ::HighPass`.
pub(crate) const KIND_RBJ_HIGH_PASS: i32 = 3;

/// `oracle_filter_design`: one filter design.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ShimFilterDesign {
    pub(crate) kind: i32,
    pub(crate) order: i32,
    pub(crate) sample_rate_hz: f64,
    pub(crate) frequency_hz: f64,
    pub(crate) width_hz: f64,
    pub(crate) quality: f64,
}

/// `oracle_compressor_settings`: the settings of one SimpleComp.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ShimCompressorSettings {
    pub(crate) sample_rate_hz: f64,
    pub(crate) threshold_db: f64,
    pub(crate) ratio: f64,
    pub(crate) attack_ms: f64,
    pub(crate) release_ms: f64,
}

/// `oracle_vector`: F3DAudio's three-float vector.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ShimVector {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
}

/// `oracle_spatial_scene`: a listener, a mono emitter, an optional curve and an optional cone.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ShimSpatialScene {
    pub(crate) listener_position: ShimVector,
    pub(crate) listener_front: ShimVector,
    pub(crate) listener_top: ShimVector,
    pub(crate) emitter_position: ShimVector,
    pub(crate) emitter_front: ShimVector,
    pub(crate) emitter_top: ShimVector,
    pub(crate) curve_distances: [f32; SHIM_MAX_CURVE_POINTS],
    pub(crate) curve_gains: [f32; SHIM_MAX_CURVE_POINTS],
    pub(crate) curve_point_count: u32,
    pub(crate) curve_distance_scaler: f32,
    pub(crate) has_cone: i32,
    pub(crate) cone_inner_angle: f32,
    pub(crate) cone_outer_angle: f32,
    pub(crate) cone_inner_volume: f32,
    pub(crate) cone_outer_volume: f32,
}

/// `oracle_spatial_result`: what F3DAudio computed for a scene.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ShimSpatialResult {
    pub(crate) left: f32,
    pub(crate) right: f32,
    pub(crate) volume: f32,
    pub(crate) emitter_to_listener_angle: f32,
    pub(crate) emitter_to_listener_distance: f32,
    pub(crate) assertion_failures: u32,
}

/// `oracle_struct_sizes`: the shim's `sizeof` of every shared struct, read by the layout tests.
#[cfg(test)]
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ShimStructSizes {
    pub(crate) filter_design: usize,
    pub(crate) compressor_settings: usize,
    pub(crate) vector: usize,
    pub(crate) spatial_scene: usize,
    pub(crate) spatial_result: usize,
}

unsafe extern "C" {
    #[cfg(test)]
    safe fn oracle_shim_struct_sizes() -> ShimStructSizes;
    fn oracle_filter_process_f32(
        design: *const ShimFilterDesign,
        channels: u32,
        samples: *mut f32,
        frames: usize,
    ) -> i32;
    fn oracle_filter_magnitudes(
        design: *const ShimFilterDesign,
        frequencies_hz: *const f64,
        magnitudes: *mut f64,
        count: usize,
    ) -> i32;
    fn oracle_filter_process_stereo_i16(
        design: *const ShimFilterDesign,
        samples: *mut i16,
        frames: usize,
        gain: f32,
    ) -> i32;
    fn oracle_simple_comp_process(
        settings: *const ShimCompressorSettings,
        samples: *mut f64,
        frames: usize,
        gains: *mut f64,
    ) -> i32;
    fn oracle_simple_comp_process_stereo_i16(
        settings: *const ShimCompressorSettings,
        samples: *mut i16,
        frames: usize,
    ) -> i32;
    fn oracle_f3daudio_calculate(
        scene: *const ShimSpatialScene,
        result: *mut ShimSpatialResult,
    ) -> i32;
}

/// The shim's `sizeof` of every shared struct.
#[cfg(test)]
pub(crate) fn struct_sizes() -> ShimStructSizes {
    oracle_shim_struct_sizes()
}

/// Turns a shim status into a result.
fn check_status(operation: &'static str, status: i32) -> Result<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(Error::OracleRefused { operation, status })
    }
}

/// The number of whole `channels`-channel frames in `samples`.
fn whole_frames(samples: usize, channels: usize) -> Result<usize> {
    if channels == 0 || !samples.is_multiple_of(channels) {
        return Err(Error::UnevenFrames { samples, channels });
    }
    Ok(samples / channels)
}

/// Filters interleaved `channels`-channel `f32` samples in place through a fresh DSPFilters
/// `SimpleFilter`.
pub(crate) fn filter_process_f32(
    design: &ShimFilterDesign,
    channels: u32,
    samples: &mut [f32],
) -> Result<()> {
    let frames = whole_frames(samples.len(), usize::try_from(channels).unwrap_or(0))?;
    // SAFETY: `design` is a live reference; `samples` is a live, exclusively borrowed slice of
    // `frames * channels` floats, which is exactly what the shim reads and writes. The shim
    // keeps no pointer after it returns.
    let status =
        unsafe { oracle_filter_process_f32(design, channels, samples.as_mut_ptr(), frames) };
    check_status("oracle_filter_process_f32", status)
}

/// Writes the filter's linear gain at each of `frequencies_hz` into `magnitudes`.
pub(crate) fn filter_magnitudes(
    design: &ShimFilterDesign,
    frequencies_hz: &[f64],
    magnitudes: &mut [f64],
) -> Result<()> {
    if frequencies_hz.len() != magnitudes.len() {
        return Err(Error::LengthMismatch {
            reference: frequencies_hz.len(),
            candidate: magnitudes.len(),
        });
    }
    // SAFETY: both slices are live and hold `frequencies_hz.len()` values, the count the shim
    // reads from the first and writes into the second; `magnitudes` is borrowed exclusively.
    let status = unsafe {
        oracle_filter_magnitudes(
            design,
            frequencies_hz.as_ptr(),
            magnitudes.as_mut_ptr(),
            frequencies_hz.len(),
        )
    };
    check_status("oracle_filter_magnitudes", status)
}

/// Filters interleaved stereo `i16` samples in place as the original `processFilterStereo` does.
pub(crate) fn filter_process_stereo_i16(
    design: &ShimFilterDesign,
    samples: &mut [i16],
    gain: f32,
) -> Result<()> {
    let frames = whole_frames(samples.len(), 2)?;
    // SAFETY: `samples` is a live, exclusively borrowed slice of `frames * 2` samples, exactly
    // what the shim reads and writes; `design` is a live reference.
    let status =
        unsafe { oracle_filter_process_stereo_i16(design, samples.as_mut_ptr(), frames, gain) };
    check_status("oracle_filter_process_stereo_i16", status)
}

/// Compresses interleaved stereo `f64` samples in place through a fresh SimpleComp and writes
/// each frame's linear gain into `gains`.
pub(crate) fn simple_comp_process(
    settings: &ShimCompressorSettings,
    samples: &mut [f64],
    gains: &mut [f64],
) -> Result<()> {
    let frames = whole_frames(samples.len(), 2)?;
    if gains.len() != frames {
        return Err(Error::LengthMismatch {
            reference: frames,
            candidate: gains.len(),
        });
    }
    // SAFETY: `samples` holds `frames * 2` doubles and `gains` holds `frames`, the counts the
    // shim writes; both are live and borrowed exclusively, and `settings` is a live reference.
    let status = unsafe {
        oracle_simple_comp_process(settings, samples.as_mut_ptr(), frames, gains.as_mut_ptr())
    };
    check_status("oracle_simple_comp_process", status)
}

/// Compresses interleaved stereo `i16` samples in place as the original `processCompressor`
/// does.
pub(crate) fn simple_comp_process_stereo_i16(
    settings: &ShimCompressorSettings,
    samples: &mut [i16],
) -> Result<()> {
    let frames = whole_frames(samples.len(), 2)?;
    // SAFETY: `samples` is a live, exclusively borrowed slice of `frames * 2` samples, exactly
    // what the shim reads and writes; `settings` is a live reference.
    let status =
        unsafe { oracle_simple_comp_process_stereo_i16(settings, samples.as_mut_ptr(), frames) };
    check_status("oracle_simple_comp_process_stereo_i16", status)
}

/// Runs `F3DAudioCalculate` for `scene`.
pub(crate) fn f3daudio_calculate(scene: &ShimSpatialScene) -> Result<ShimSpatialResult> {
    let mut result = ShimSpatialResult::default();
    // SAFETY: `scene` is a live reference whose curve arrays hold `SHIM_MAX_CURVE_POINTS` floats,
    // and the shim refuses a larger `curve_point_count`; `result` is a live local the shim only
    // writes.
    let status = unsafe { oracle_f3daudio_calculate(scene, &raw mut result) };
    check_status("oracle_f3daudio_calculate", status)?;
    Ok(result)
}
