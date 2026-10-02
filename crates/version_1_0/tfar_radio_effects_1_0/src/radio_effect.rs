//! `RadioEffect`: the interface every 1.0 radio effect offers.

/// The sample rate every effect is designed for: TeamSpeak's 48 kHz voice.
pub const SAMPLE_RATE_HZ: f64 = 48_000.0;

/// One radio effect chain over a mono `f32` block at [`SAMPLE_RATE_HZ`].
///
/// An effect keeps state from block to block (delay line, filter memory, ring-modulator phase,
/// generator), so each sender and radio owns its own instance. Create it off the TeamSpeak audio
/// thread; [`RadioEffect::process`] runs on it and allocates nothing.
pub trait RadioEffect {
    /// Sets the signal degradation from the raw distance error (the sender's distance over the
    /// radio's range, the value the original hands `setErrorLeveL`). The speaker effects map it
    /// through [`crate::error_level_from_distance`]; the underwater effect uses it as the dropout
    /// probability directly.
    fn set_error_level(&mut self, distance_error: f64);

    /// Runs the effect over `block` in place. An empty block is left alone.
    fn process(&mut self, block: &mut [f32]);
}
