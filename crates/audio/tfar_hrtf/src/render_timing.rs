//! The sample rate, block sizes and latency every HRTF source renders with.
//!
//! **Role:** fixes the one set of numbers the sphere loader, the `hrtf` processor and the block
//! adapter must agree on, so the latency is a documented constant rather than an accident.
//!
//! **Position:** a module of `tfar_hrtf`; read by `hrir_library` (resampling, processor shape) and
//! `hrtf_source` (adapter size); exported for the voice pipelines, which report the latency.
//!
//! **Signals and state:** constants only.
//!
//! **Invariants:** [`RENDER_BLOCK_FRAMES`] is `INTERPOLATION_STEPS * CONVOLUTION_BLOCK_FRAMES`, and
//! the added latency equals it: 480 frames, 10 ms at 48 kHz.

/// The sample rate TeamSpeak's voice callbacks run at, and the rate the sphere is resampled to.
pub const SAMPLE_RATE_HZ: u32 = 48_000;

/// The frames of one overlap-save convolution step inside the `hrtf` processor.
///
/// 120 frames (2.5 ms) keeps the FFT short (`120 + HRIR length - 1` points) while the direction is
/// re-sampled every 2.5 ms.
pub const CONVOLUTION_BLOCK_FRAMES: usize = 120;

/// The convolution steps per rendered block. The `hrtf` processor moves the direction and the
/// distance gain linearly from the previous block's values to the new ones across these steps, so
/// a moving source changes smoothly.
pub const INTERPOLATION_STEPS: usize = 4;

/// The frames the `hrtf` processor renders at once: `INTERPOLATION_STEPS * CONVOLUTION_BLOCK_FRAMES`
/// = 480 frames, 10 ms at 48 kHz, the length of TeamSpeak's most common voice frame.
pub const RENDER_BLOCK_FRAMES: usize = INTERPOLATION_STEPS * CONVOLUTION_BLOCK_FRAMES;

/// The delay the block adapter adds, in frames: exactly one render block, 480 frames (10 ms),
/// whatever frame sizes TeamSpeak delivers. Output frame `n` carries input frame `n - 480`; the
/// first 480 output frames of a source (and of a source after `reset`) are silence.
///
/// The head-related impulse responses add their own acoustic onset on top (the sound reaching
/// the far ear later), which is part of the binaural effect and not counted here.
pub const ADDED_LATENCY_FRAMES: usize = RENDER_BLOCK_FRAMES;
