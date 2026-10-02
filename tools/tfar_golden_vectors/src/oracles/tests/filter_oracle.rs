//! Unit tests for `filter_oracle`: parameter checks and the plain properties of the originals.

use super::*;

const RATE: f64 = 48_000.0;

fn low_pass(order: usize, cutoff_hz: f64) -> Result<OracleFilter> {
    OracleFilter::new(
        OracleFilterDesign::ButterworthLowPass { order, cutoff_hz },
        RATE,
    )
}

#[test]
fn an_order_outside_the_instantiated_range_is_refused() {
    for order in [0, ORACLE_MAX_BUTTERWORTH_ORDER + 1] {
        assert!(matches!(
            low_pass(order, 1_000.0),
            Err(Error::InvalidParameter {
                parameter: "order",
                ..
            })
        ));
    }
}

#[test]
fn a_cutoff_at_nyquist_and_a_zero_quality_are_refused() {
    assert!(low_pass(2, RATE / 2.0).is_err());
    let rbj = OracleFilter::new(
        OracleFilterDesign::RbjHighPass {
            cutoff_hz: 500.0,
            quality: 0.0,
        },
        RATE,
    );
    assert!(rbj.is_err());
}

#[test]
fn a_butterworth_low_pass_is_unity_at_dc_and_three_decibels_down_at_its_cutoff() {
    let filter = low_pass(4, 1_000.0).unwrap();
    let magnitudes = filter.magnitudes(&[1.0e-6, 1_000.0]).unwrap();
    assert!((magnitudes[0] - 1.0).abs() < 1.0e-9);
    assert!((magnitudes[1] - core::f64::consts::FRAC_1_SQRT_2).abs() < 1.0e-6);
}

#[test]
fn a_low_pass_settles_to_the_level_of_a_constant_input() {
    let filter = low_pass(2, 2_000.0).unwrap();
    let mut samples = vec![0.5_f32; 4_800];
    filter.process_mono_f32(&mut samples).unwrap();
    assert!((samples[4_799] - 0.5).abs() < 1.0e-5);
}

#[test]
fn every_processing_call_starts_from_silent_state() {
    let filter = low_pass(2, 2_000.0).unwrap();
    let mut first = vec![[0.25_f32, -0.25]; 64];
    let mut second = first.clone();
    filter.process_stereo_f32(&mut first).unwrap();
    filter.process_stereo_f32(&mut second).unwrap();
    assert_eq!(first, second);
}

#[test]
fn the_stereo_i16_path_clips_to_the_full_i16_range_as_the_original_does() {
    let filter = low_pass(2, 20_000.0).unwrap();
    let mut samples = vec![i16::MAX, i16::MIN];
    samples.extend(core::iter::repeat_n([i16::MAX, i16::MIN], 999).flatten());
    filter.process_stereo_i16(&mut samples, 4.0).unwrap();
    assert_eq!(samples[1998], i16::MAX);
    assert_eq!(samples[1999], i16::MIN);
}
