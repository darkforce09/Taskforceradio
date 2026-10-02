//! The one-pole attack and release envelope follower of `SimpleSource`.
//!
//! The specification is `EnvelopeDetector` and `AttRelEnvelope` in
//! `reference/tfar_1_0/ts/include/simpleSource/SimpleEnvelope.{h,cpp}`: a time constant of `ms`
//! milliseconds at `sample_rate` gives the coefficient `exp(-1000 / (ms * sample_rate))`, and each
//! step moves the state toward the input by `state = input + coefficient * (state - input)`, with
//! the attack coefficient when the input is above the state and the release coefficient
//! otherwise. Plain state, run on the TeamSpeak audio thread; nothing allocates.

#[cfg(test)]
#[path = "tests/attack_release_envelope.rs"]
mod tests;

/// The coefficient of a one-pole follower with a time constant of `milliseconds` at
/// `sample_rate_hz`: after that many milliseconds a step has covered `1 - 1/e` of its distance.
pub(crate) fn time_constant_coefficient(milliseconds: f64, sample_rate_hz: f64) -> f64 {
    (-1000.0 / (milliseconds * sample_rate_hz)).exp()
}

/// The attack and release coefficients of one envelope; the state lives with the caller, as the
/// original passes it by reference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AttackReleaseEnvelope {
    attack_coefficient: f64,
    release_coefficient: f64,
}

impl AttackReleaseEnvelope {
    /// An envelope with the given time constants in milliseconds at `sample_rate_hz`.
    pub(crate) fn new(attack_ms: f64, release_ms: f64, sample_rate_hz: f64) -> Self {
        Self {
            attack_coefficient: time_constant_coefficient(attack_ms, sample_rate_hz),
            release_coefficient: time_constant_coefficient(release_ms, sample_rate_hz),
        }
    }

    /// Moves `state` one sample toward `input`: attack when `input > state`, release otherwise.
    pub(crate) fn run(&self, input: f64, state: &mut f64) {
        let coefficient = if input > *state {
            self.attack_coefficient
        } else {
            self.release_coefficient
        };
        *state = input + coefficient * (*state - input);
    }
}
