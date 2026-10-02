//! The coefficients of a designed filter: a cascade of normalised second-order sections.
//!
//! **Role:** holds the biquad coefficients a [`crate::FilterDesign`] produces, together with the
//! sample rate they were designed for, and evaluates the cascade's magnitude response.
//!
//! **Position:** tier 1 of crates/audio; built by the Butterworth and RBJ designs in
//! `designs/`, consumed by [`crate::ChannelFilter`], read by tests and the golden-vector
//! comparison.
//!
//! **Signals and state:** plain values; built off the audio thread, read anywhere.
//!
//! **Invariants:** every section is normalised (`a0 = 1`) and every coefficient is finite; a
//! cascade holds at least one section.

use crate::error::{Error, Result};

#[cfg(test)]
#[path = "tests/second_order_sections.rs"]
mod tests;

/// One biquad section `H(z) = (b0 + b1 z^-1 + b2 z^-2) / (1 + a1 z^-1 + a2 z^-2)`, normalised so
/// that the leading denominator coefficient is 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SecondOrderSection {
    /// The numerator coefficient of `z^0`.
    pub b0: f64,
    /// The numerator coefficient of `z^-1`.
    pub b1: f64,
    /// The numerator coefficient of `z^-2`.
    pub b2: f64,
    /// The denominator coefficient of `z^-1`.
    pub a1: f64,
    /// The denominator coefficient of `z^-2`.
    pub a2: f64,
}

impl SecondOrderSection {
    /// Normalises a section given as numerator `b` and denominator `a` (`a[0]` the leading
    /// coefficient), dividing every coefficient by `a[0]`.
    ///
    /// # Errors
    ///
    /// [`Error::DesignFailed`] when `a[0]` is zero or a normalised coefficient is not finite.
    pub fn from_unnormalised(b: [f64; 3], a: [f64; 3]) -> Result<Self> {
        let [b0, b1, b2] = b;
        let [a0, a1, a2] = a;
        if a0 == 0.0 {
            return Err(Error::DesignFailed {
                reason: "a section has a zero leading denominator coefficient",
            });
        }
        let section = Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        };
        if section.coefficients().iter().all(|value| value.is_finite()) {
            Ok(section)
        } else {
            Err(Error::DesignFailed {
                reason: "a section has a coefficient that is not finite",
            })
        }
    }

    /// The five coefficients in the order `b0, b1, b2, a1, a2`.
    pub fn coefficients(&self) -> [f64; 5] {
        [self.b0, self.b1, self.b2, self.a1, self.a2]
    }

    /// The section's gain `|H(e^{jω})|` at the normalised angular frequency `omega`
    /// (radians per sample).
    fn magnitude_at_angle(&self, omega: f64) -> f64 {
        let (sin_1, cos_1) = omega.sin_cos();
        let (sin_2, cos_2) = (2.0 * omega).sin_cos();
        let numerator_real = self.b0 + self.b1 * cos_1 + self.b2 * cos_2;
        let numerator_imaginary = -(self.b1 * sin_1 + self.b2 * sin_2);
        let denominator_real = 1.0 + self.a1 * cos_1 + self.a2 * cos_2;
        let denominator_imaginary = -(self.a1 * sin_1 + self.a2 * sin_2);
        numerator_real.hypot(numerator_imaginary) / denominator_real.hypot(denominator_imaginary)
    }
}

/// A designed filter: a cascade of [`SecondOrderSection`]s run one after the other, at the sample
/// rate it was designed for.
#[derive(Debug, Clone, PartialEq)]
pub struct SecondOrderSections {
    sections: Vec<SecondOrderSection>,
    sample_rate_hz: f64,
}

impl SecondOrderSections {
    /// A cascade of `sections` designed for `sample_rate_hz`. Allocates nothing beyond the given
    /// `Vec`.
    ///
    /// # Errors
    ///
    /// [`Error::DesignFailed`] when `sections` is empty; [`Error::InvalidSampleRate`] when the
    /// sample rate is not a finite number above zero.
    pub fn new(sections: Vec<SecondOrderSection>, sample_rate_hz: f64) -> Result<Self> {
        crate::designs::check_sample_rate(sample_rate_hz)?;
        if sections.is_empty() {
            return Err(Error::DesignFailed {
                reason: "the design holds no section",
            });
        }
        Ok(Self {
            sections,
            sample_rate_hz,
        })
    }

    /// The sections in processing order.
    pub fn sections(&self) -> &[SecondOrderSection] {
        &self.sections
    }

    /// The sample rate the coefficients were designed for, in hertz.
    pub fn sample_rate_hz(&self) -> f64 {
        self.sample_rate_hz
    }

    /// The cascade's linear gain at `frequency_hz` (0 is DC, half the sample rate is Nyquist):
    /// the product of every section's `|H(e^{jω})|`.
    pub fn magnitude_at(&self, frequency_hz: f64) -> f64 {
        let omega = core::f64::consts::TAU * frequency_hz / self.sample_rate_hz;
        self.sections
            .iter()
            .map(|section| section.magnitude_at_angle(omega))
            .product()
    }
}
