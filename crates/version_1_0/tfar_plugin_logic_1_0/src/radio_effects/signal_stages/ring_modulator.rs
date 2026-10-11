//! `RingModulator`: the error-scaled ring modulation of the speaker effects (`ringmodulation`,
//! `reference/tfar_1_0/ts/src/RadioEffect.hpp:160`).

#[cfg(test)]
#[path = "tests/ring_modulator.rs"]
mod tests;

/// The phase advance per sample: 90 / 48000, so the phase ramp repeats at about 90 Hz.
const PHASE_STEP: f32 = 90.0 / 48_000.0;

/// A ring modulator whose carrier is `sin(phase · π/2)`, the phase ramping from 0 and starting
/// over once it passes 1.
#[derive(Debug, Clone, Default)]
pub(crate) struct RingModulator {
    phase: f32,
}

impl RingModulator {
    /// Mixes `input` with its modulated copy: `in · (1 − mix) + in · sin(phase · π/2) · mix`, then
    /// advances the phase.
    pub(crate) fn modulate(&mut self, input: f32, mix: f32) -> f32 {
        let multiple = input * (self.phase * core::f32::consts::FRAC_PI_2).sin();
        self.phase += PHASE_STEP;
        if self.phase > 1.0 {
            self.phase = 0.0;
        }
        input * (1.0 - mix) + multiple * mix
    }
}
