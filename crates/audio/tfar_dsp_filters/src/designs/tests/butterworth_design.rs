//! Unit tests for the Butterworth designs: gains at DC, Nyquist and the −3 dB points, the
//! band-edge mapping, and the analytic Butterworth magnitude over the whole band.

use super::*;

const SAMPLE_RATE_HZ: f64 = 48_000.0;
const HALF_POWER: f64 = core::f64::consts::FRAC_1_SQRT_2;

/// `tan(π f / fs)`: the bilinear-warped frequency the digital Butterworth magnitude is a function
/// of.
fn warped(frequency_hz: f64) -> f64 {
    (core::f64::consts::PI * frequency_hz / SAMPLE_RATE_HZ).tan()
}

/// The digital Butterworth low-pass magnitude `1 / sqrt(1 + (Ω / Ωc)^(2N))`.
fn analytic_low_pass(order: usize, cutoff_hz: f64, frequency_hz: f64) -> f64 {
    let ratio = warped(frequency_hz) / warped(cutoff_hz);
    1.0 / (1.0 + ratio.powi(2 * i32::try_from(order).unwrap())).sqrt()
}

/// The digital Butterworth band-pass magnitude `1 / sqrt(1 + ((Ω² − Ω1 Ω2) / (Ω (Ω2 − Ω1)))^(2N))`
/// with the band edges `Ω1`, `Ω2`.
fn analytic_band_pass(order: usize, lower_hz: f64, upper_hz: f64, frequency_hz: f64) -> f64 {
    let (lower, upper, omega) = (warped(lower_hz), warped(upper_hz), warped(frequency_hz));
    let ratio = (omega * omega - lower * upper) / (omega * (upper - lower));
    1.0 / (1.0 + ratio.powi(2 * i32::try_from(order).unwrap())).sqrt()
}

/// The frequency where the band-pass peaks: the warped geometric centre of the band edges, as
/// DSPFilters normalises its band-pass (`2 atan(sqrt(tan(wc / 2) tan(wc2 / 2)))`).
fn warped_centre_hz(lower_hz: f64, upper_hz: f64) -> f64 {
    (warped(lower_hz) * warped(upper_hz)).sqrt().atan() * SAMPLE_RATE_HZ / core::f64::consts::PI
}

fn sweep() -> impl Iterator<Item = f64> {
    (1..240).map(|step| f64::from(step) * 100.0 - 50.0)
}

#[test]
fn the_low_pass_has_unit_gain_at_dc_and_none_at_nyquist() {
    for (order, cutoff_hz) in [(2, 5_000.0), (2, 2_000.0), (4, 100.0), (1, 3_000.0)] {
        let sections = butterworth_low_pass(order, cutoff_hz, SAMPLE_RATE_HZ).unwrap();
        assert!(
            (sections.magnitude_at(0.0) - 1.0).abs() < 1e-9,
            "{order} {cutoff_hz}"
        );
        assert!(
            sections.magnitude_at(SAMPLE_RATE_HZ / 2.0) < 1e-9,
            "{order} {cutoff_hz}"
        );
    }
}

#[test]
fn the_low_pass_is_three_db_down_at_its_cutoff() {
    for (order, cutoff_hz) in [(2, 5_000.0), (2, 1_200.0), (4, 100.0), (4, 8_000.0)] {
        let sections = butterworth_low_pass(order, cutoff_hz, SAMPLE_RATE_HZ).unwrap();
        assert!((sections.magnitude_at(cutoff_hz) - HALF_POWER).abs() < 1e-9);
    }
}

#[test]
fn the_low_pass_follows_the_analytic_butterworth_magnitude() {
    for (order, cutoff_hz) in [(2, 5_000.0), (4, 100.0), (4, 3_000.0)] {
        let sections = butterworth_low_pass(order, cutoff_hz, SAMPLE_RATE_HZ).unwrap();
        for frequency_hz in sweep() {
            let expected = analytic_low_pass(order, cutoff_hz, frequency_hz);
            let actual = sections.magnitude_at(frequency_hz);
            assert!(
                (actual - expected).abs() < 1e-9,
                "order {order}, cutoff {cutoff_hz}, at {frequency_hz}: {actual} vs {expected}"
            );
        }
    }
}

#[test]
fn an_order_n_low_pass_has_n_poles_in_ceil_n_over_two_sections() {
    for (order, sections) in [(1, 1), (2, 1), (3, 2), (4, 2), (8, 4)] {
        let design = butterworth_low_pass(order, 1_000.0, SAMPLE_RATE_HZ).unwrap();
        assert_eq!(design.sections().len(), sections, "order {order}");
    }
}

#[test]
fn the_band_edges_are_the_centre_minus_and_plus_half_the_width() {
    assert_eq!(band_edges_hz(2_000.0, 1_000.0), (1_500.0, 2_500.0));
    assert_eq!(band_edges_hz(1_850.0, 1_550.0), (1_075.0, 2_625.0));
    assert_eq!(band_edges_hz(1_000.0, 400.0), (800.0, 1_200.0));
}

#[test]
fn the_band_pass_is_three_db_down_at_both_band_edges() {
    for (order, centre_hz, width_hz) in [
        (1, 2_000.0, 1_000.0),
        (2, 1_850.0, 1_550.0),
        (2, 1_000.0, 400.0),
    ] {
        let sections = butterworth_band_pass(order, centre_hz, width_hz, SAMPLE_RATE_HZ).unwrap();
        let (lower_hz, upper_hz) = band_edges_hz(centre_hz, width_hz);
        assert!((sections.magnitude_at(lower_hz) - HALF_POWER).abs() < 1e-9);
        assert!((sections.magnitude_at(upper_hz) - HALF_POWER).abs() < 1e-9);
    }
}

#[test]
fn the_band_pass_peaks_at_unit_gain_near_its_centre() {
    for (order, centre_hz, width_hz) in [
        (1, 2_000.0, 1_000.0),
        (2, 1_850.0, 1_550.0),
        (2, 1_000.0, 400.0),
    ] {
        let sections = butterworth_band_pass(order, centre_hz, width_hz, SAMPLE_RATE_HZ).unwrap();
        let (lower_hz, upper_hz) = band_edges_hz(centre_hz, width_hz);
        let peak_hz = warped_centre_hz(lower_hz, upper_hz);
        assert!((sections.magnitude_at(peak_hz) - 1.0).abs() < 1e-9);
        // No frequency of a 1 Hz sweep over the band is louder than the peak.
        let loudest = (1..24_000)
            .map(|frequency| sections.magnitude_at(f64::from(frequency)))
            .fold(0.0_f64, f64::max);
        assert!(loudest <= 1.0 + 1e-9, "{loudest}");
        // A geometric centre lies below the arithmetic one, inside the band.
        assert!(
            lower_hz < peak_hz && peak_hz < centre_hz,
            "{peak_hz} vs {centre_hz}"
        );
        let centre_gain = sections.magnitude_at(centre_hz);
        assert!(centre_gain > 0.9, "{centre_gain} at {centre_hz}");
        assert!(sections.magnitude_at(0.0) < 1e-9);
        assert!(sections.magnitude_at(SAMPLE_RATE_HZ / 2.0) < 1e-9);
    }
}

#[test]
fn the_band_pass_follows_the_analytic_butterworth_magnitude() {
    for (order, centre_hz, width_hz) in [
        (1, 2_000.0, 1_000.0),
        (2, 1_850.0, 1_550.0),
        (2, 1_000.0, 400.0),
    ] {
        let sections = butterworth_band_pass(order, centre_hz, width_hz, SAMPLE_RATE_HZ).unwrap();
        let (lower_hz, upper_hz) = band_edges_hz(centre_hz, width_hz);
        for frequency_hz in sweep() {
            let expected = analytic_band_pass(order, lower_hz, upper_hz, frequency_hz);
            let actual = sections.magnitude_at(frequency_hz);
            assert!(
                (actual - expected).abs() < 1e-9,
                "order {order}, {centre_hz}/{width_hz}, at {frequency_hz}: {actual} vs {expected}"
            );
        }
    }
}

#[test]
fn an_order_n_band_pass_has_two_n_poles_in_n_sections() {
    for order in [1, 2, 4] {
        let design = butterworth_band_pass(order, 2_000.0, 1_000.0, SAMPLE_RATE_HZ).unwrap();
        assert_eq!(design.sections().len(), order, "order {order}");
    }
}

#[test]
fn a_band_reaching_below_zero_or_beyond_nyquist_is_refused() {
    let below = butterworth_band_pass(2, 300.0, 1_000.0, SAMPLE_RATE_HZ);
    assert!(matches!(
        below,
        Err(Error::FrequencyOutOfRange {
            frequency_role: "lower band edge",
            ..
        })
    ));
    let beyond = butterworth_band_pass(2, 23_000.0, 4_000.0, SAMPLE_RATE_HZ);
    assert!(matches!(
        beyond,
        Err(Error::FrequencyOutOfRange {
            frequency_role: "upper band edge",
            ..
        })
    ));
}

#[test]
fn a_zero_negative_tiny_or_non_finite_width_is_refused() {
    for bad in [0.0, -400.0, f64::NAN, f64::INFINITY, 1e-14] {
        let result = butterworth_band_pass(2, 1_000.0, bad, SAMPLE_RATE_HZ);
        assert!(
            matches!(result, Err(Error::InvalidBandWidth { .. })),
            "{bad}: {result:?}"
        );
    }
}

#[test]
fn a_bad_order_or_sample_rate_is_refused_before_designing() {
    assert!(matches!(
        butterworth_low_pass(0, 1_000.0, SAMPLE_RATE_HZ),
        Err(Error::InvalidOrder { .. })
    ));
    assert!(matches!(
        butterworth_band_pass(9, 1_000.0, 400.0, SAMPLE_RATE_HZ),
        Err(Error::InvalidOrder { .. })
    ));
    assert!(matches!(
        butterworth_low_pass(2, 1_000.0, f64::NAN),
        Err(Error::InvalidSampleRate { .. })
    ));
}
