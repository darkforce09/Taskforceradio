//! Unit tests for the RBJ designs: the cookbook coefficients DSPFilters computes, and the gains at
//! DC, Nyquist and the corner.

use super::*;

const SAMPLE_RATE_HZ: f64 = 48_000.0;

/// The TFAR radio-effect corners: (high-pass cutoff, Q), (low-pass cutoff, Q) of the LR,
/// airborne, SW and 0.9 SW speaker filters.
const RADIO_EFFECT_CORNERS: [(f64, f64); 7] = [
    (520.0, 0.97),
    (1_300.0, 1.0),
    (1_000.0, 1.0),
    (4_000.0, 1.0),
    (900.0, 0.85),
    (3_000.0, 2.0),
    (4_300.0, 2.0),
];

/// The coefficients `DSPFilters::RBJ::LowPass::setup` or `HighPass::setup` computes, normalised
/// by `a0` as `BiquadBase::setCoefficients` does.
fn dsp_filters_coefficients(is_low_pass: bool, cutoff_hz: f64, quality: f64) -> [f64; 5] {
    let w0 = 2.0 * core::f64::consts::PI * cutoff_hz / SAMPLE_RATE_HZ;
    let (sn, cs) = w0.sin_cos();
    let alpha = sn / (2.0 * quality);
    let (b0, b1, b2) = if is_low_pass {
        ((1.0 - cs) / 2.0, 1.0 - cs, (1.0 - cs) / 2.0)
    } else {
        (f64::midpoint(1.0, cs), -(1.0 + cs), f64::midpoint(1.0, cs))
    };
    let (a0, a1, a2) = (1.0 + alpha, -2.0 * cs, 1.0 - alpha);
    [b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0]
}

fn assert_close(actual: [f64; 5], expected: [f64; 5]) {
    for (a, e) in actual.iter().zip(expected) {
        assert!((a - e).abs() < 1e-12, "{actual:?} vs {expected:?}");
    }
}

#[test]
fn the_low_pass_coefficients_are_the_ones_dsp_filters_computes() {
    for (cutoff_hz, quality) in RADIO_EFFECT_CORNERS {
        let sections = rbj_low_pass(cutoff_hz, quality, SAMPLE_RATE_HZ).unwrap();
        let [section] = sections.sections() else {
            panic!("one section")
        };
        assert_close(
            section.coefficients(),
            dsp_filters_coefficients(true, cutoff_hz, quality),
        );
    }
}

#[test]
fn the_high_pass_coefficients_are_the_ones_dsp_filters_computes() {
    for (cutoff_hz, quality) in RADIO_EFFECT_CORNERS {
        let sections = rbj_high_pass(cutoff_hz, quality, SAMPLE_RATE_HZ).unwrap();
        let [section] = sections.sections() else {
            panic!("one section")
        };
        assert_close(
            section.coefficients(),
            dsp_filters_coefficients(false, cutoff_hz, quality),
        );
    }
}

#[test]
fn the_low_pass_passes_dc_blocks_nyquist_and_has_gain_q_at_the_corner() {
    for (cutoff_hz, quality) in RADIO_EFFECT_CORNERS {
        let sections = rbj_low_pass(cutoff_hz, quality, SAMPLE_RATE_HZ).unwrap();
        assert!((sections.magnitude_at(0.0) - 1.0).abs() < 1e-12);
        assert!(sections.magnitude_at(SAMPLE_RATE_HZ / 2.0) < 1e-12);
        assert!((sections.magnitude_at(cutoff_hz) - quality).abs() < 1e-9);
    }
}

#[test]
fn the_high_pass_blocks_dc_passes_nyquist_and_has_gain_q_at_the_corner() {
    for (cutoff_hz, quality) in RADIO_EFFECT_CORNERS {
        let sections = rbj_high_pass(cutoff_hz, quality, SAMPLE_RATE_HZ).unwrap();
        assert!(sections.magnitude_at(0.0) < 1e-12);
        assert!((sections.magnitude_at(SAMPLE_RATE_HZ / 2.0) - 1.0).abs() < 1e-12);
        assert!((sections.magnitude_at(cutoff_hz) - quality).abs() < 1e-9);
    }
}

#[test]
fn bad_rbj_parameters_are_refused() {
    assert!(matches!(
        rbj_low_pass(0.0, 1.0, SAMPLE_RATE_HZ),
        Err(Error::FrequencyOutOfRange { .. })
    ));
    assert!(matches!(
        rbj_high_pass(24_000.0, 1.0, SAMPLE_RATE_HZ),
        Err(Error::FrequencyOutOfRange { .. })
    ));
    assert!(matches!(
        rbj_low_pass(1_000.0, 0.0, SAMPLE_RATE_HZ),
        Err(Error::InvalidQuality { .. })
    ));
    assert!(matches!(
        rbj_high_pass(1_000.0, f64::NAN, SAMPLE_RATE_HZ),
        Err(Error::InvalidQuality { .. })
    ));
    assert!(matches!(
        rbj_low_pass(1_000.0, 1.0, -1.0),
        Err(Error::InvalidSampleRate { .. })
    ));
}
