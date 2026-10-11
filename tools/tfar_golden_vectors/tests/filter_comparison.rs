//! `tfar_audio::dsp_filters` against DSPFilters: every filter design the two mods use, over white noise,
//! a sine sweep and an impulse at 48 kHz, in mono, stereo and the stereo `i16` path, plus the
//! magnitude response.

#![allow(
    clippy::cast_possible_truncation,
    reason = "the oracle and the filters run in f32 and i16, so the f64 test signals narrow on purpose"
)]
#![allow(
    clippy::unwrap_used,
    reason = "helpers outside the test functions stop the test on a failed call, as the tests do"
)]

use tfar_audio::dsp_filters::{FilterDesign, MonoFilter, StereoFilter};
use tfar_audio::sample_buffer::StereoBuffer;
use tfar_golden_vectors::{
    OracleFilter, OracleFilterDesign, SAMPLE_RATE_HZ, decibel_difference, impulse,
    log_spaced_frequencies, max_absolute_difference, relative_rms_error, sine_sweep, white_noise,
};

/// The RMS error a filtered signal may have, relative to the oracle's RMS.
const RELATIVE_RMS_TOLERANCE: f64 = 1.0e-4;
/// The magnitude tolerance where the oracle's gain is at or above −3 dB.
const PASSBAND_TOLERANCE_DB: f64 = 0.1;
/// The magnitude tolerance everywhere else.
const STOPBAND_TOLERANCE_DB: f64 = 1.0;

/// One design as `tfar_audio::dsp_filters` names it and as DSPFilters sets it up.
struct DesignCase {
    name: &'static str,
    ours: FilterDesign,
    original: OracleFilterDesign,
}

fn butterworth_low_pass(name: &'static str, order: usize, cutoff_hz: f64) -> DesignCase {
    DesignCase {
        name,
        ours: FilterDesign::ButterworthLowPass { order, cutoff_hz },
        original: OracleFilterDesign::ButterworthLowPass { order, cutoff_hz },
    }
}

fn butterworth_band_pass(
    name: &'static str,
    order: usize,
    centre_hz: f64,
    width_hz: f64,
) -> DesignCase {
    DesignCase {
        name,
        ours: FilterDesign::ButterworthBandPass {
            order,
            centre_hz,
            width_hz,
        },
        original: OracleFilterDesign::ButterworthBandPass {
            order,
            centre_hz,
            width_hz,
        },
    }
}

fn rbj_low_pass(name: &'static str, cutoff_hz: f64, quality: f64) -> DesignCase {
    DesignCase {
        name,
        ours: FilterDesign::RbjLowPass { cutoff_hz, quality },
        original: OracleFilterDesign::RbjLowPass { cutoff_hz, quality },
    }
}

fn rbj_high_pass(name: &'static str, cutoff_hz: f64, quality: f64) -> DesignCase {
    DesignCase {
        name,
        ours: FilterDesign::RbjHighPass { cutoff_hz, quality },
        original: OracleFilterDesign::RbjHighPass { cutoff_hz, quality },
    }
}

/// Every design the plugins set up (`reference/tfar_1_0/ts/src/clientData.hpp`, `clientData.cpp`,
/// `RadioEffect.hpp` and the 0.9.12 `plugin.cpp`, `RadioEffect.h`).
fn tfar_designs() -> Vec<DesignCase> {
    vec![
        butterworth_low_pass("vehicle, no loss: low-pass 2 at 5000 Hz", 2, 5_000.0),
        butterworth_low_pass("vehicle, half loss: low-pass 2 at 2500 Hz", 2, 2_500.0),
        butterworth_low_pass("object interception: low-pass 2 at 1400 Hz", 2, 1_400.0),
        butterworth_low_pass("can't speak: low-pass 4 at 100 Hz", 4, 100.0),
        butterworth_band_pass(
            "speaker: band-pass 1 at 2000 / 1000 Hz",
            1,
            2_000.0,
            1_000.0,
        ),
        butterworth_band_pass("phone: band-pass 2 at 1850 / 1550 Hz", 2, 1_850.0, 1_550.0),
        butterworth_band_pass("DD: band-pass 2 at 1000 / 400 Hz", 2, 1_000.0, 400.0),
        rbj_high_pass(
            "SW and LR speaker: RBJ high-pass 520 Hz, Q 0.97",
            520.0,
            0.97,
        ),
        rbj_low_pass(
            "SW and LR speaker: RBJ low-pass 1300 Hz, Q 1.0",
            1_300.0,
            1.0,
        ),
        rbj_high_pass(
            "airborne speaker: RBJ high-pass 1000 Hz, Q 1.0",
            1_000.0,
            1.0,
        ),
        rbj_low_pass(
            "airborne speaker: RBJ low-pass 4000 Hz, Q 1.0",
            4_000.0,
            1.0,
        ),
        rbj_high_pass("microphone: RBJ high-pass 900 Hz, Q 0.85", 900.0, 0.85),
        rbj_low_pass("microphone: RBJ low-pass 3000 Hz, Q 2.0", 3_000.0, 2.0),
        rbj_low_pass("0.9 LR speaker: RBJ low-pass 4300 Hz, Q 2.0", 4_300.0, 2.0),
    ]
}

fn to_f32(signal: &[f64]) -> Vec<f32> {
    signal.iter().map(|sample| *sample as f32).collect()
}

fn to_f64(signal: &[f32]) -> Vec<f64> {
    signal.iter().map(|sample| f64::from(*sample)).collect()
}

/// The relative RMS error of our mono filter against DSPFilters' over `signal`.
fn mono_error(case: &DesignCase, signal: &[f64]) -> f64 {
    let mut original = to_f32(signal);
    OracleFilter::new(case.original, SAMPLE_RATE_HZ)
        .unwrap()
        .process_mono_f32(&mut original)
        .unwrap();
    let mut ours: Vec<[f32; 1]> = to_f32(signal).into_iter().map(|sample| [sample]).collect();
    MonoFilter::new(case.ours, SAMPLE_RATE_HZ)
        .unwrap()
        .process_frames_f32(&mut ours);
    relative_rms_error(&to_f64(&original), &to_f64(ours.as_flattened())).unwrap()
}

/// Runs every design over `signal` and fails with every case above the tolerance.
fn check_every_design_on(signal_name: &str, signal: &[f64]) {
    let mut failures = Vec::new();
    let mut worst = 0.0_f64;
    for case in tfar_designs() {
        let error = mono_error(&case, signal);
        worst = worst.max(error);
        if error.is_nan() || error > RELATIVE_RMS_TOLERANCE {
            failures.push(format!("{}: relative RMS error {error:e}", case.name));
        }
    }
    println!("{signal_name}: worst relative RMS error {worst:e}");
    assert!(
        failures.is_empty(),
        "{signal_name}:\n{}",
        failures.join("\n")
    );
}

#[test]
fn every_tfar_design_matches_dspfilters_on_white_noise() {
    check_every_design_on("white noise", &white_noise(48_000, 0.5, 11));
}

#[test]
fn every_tfar_design_matches_dspfilters_on_a_sine_sweep() {
    let sweep = sine_sweep(96_000, 20.0, 20_000.0, 0.5, SAMPLE_RATE_HZ);
    check_every_design_on("sine sweep", &sweep);
}

#[test]
fn every_tfar_design_matches_dspfilters_on_an_impulse() {
    check_every_design_on("impulse", &impulse(48_000, 1.0));
}

#[test]
fn every_tfar_design_matches_dspfilters_on_independent_stereo_channels() {
    let left = white_noise(24_000, 0.5, 21);
    let right = sine_sweep(24_000, 50.0, 15_000.0, 0.5, SAMPLE_RATE_HZ);
    let frames: Vec<[f32; 2]> = left
        .iter()
        .zip(&right)
        .map(|(left, right)| [*left as f32, *right as f32])
        .collect();
    let mut failures = Vec::new();
    let mut worst = 0.0_f64;
    for case in tfar_designs() {
        let mut original = frames.clone();
        OracleFilter::new(case.original, SAMPLE_RATE_HZ)
            .unwrap()
            .process_stereo_f32(&mut original)
            .unwrap();
        let mut ours = frames.clone();
        StereoFilter::new(case.ours, SAMPLE_RATE_HZ)
            .unwrap()
            .process_frames_f32(&mut ours);
        for channel in 0..2 {
            let original_channel: Vec<f64> = original
                .iter()
                .map(|frame| f64::from(frame[channel]))
                .collect();
            let our_channel: Vec<f64> =
                ours.iter().map(|frame| f64::from(frame[channel])).collect();
            let error = relative_rms_error(&original_channel, &our_channel).unwrap();
            worst = worst.max(error);
            if error.is_nan() || error > RELATIVE_RMS_TOLERANCE {
                failures.push(format!("{} channel {channel}: {error:e}", case.name));
            }
        }
    }
    println!("stereo: worst relative RMS error {worst:e}");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_magnitude_response_matches_within_a_tenth_of_a_decibel_in_the_passband_and_one_elsewhere() {
    let frequencies = log_spaced_frequencies(64, 20.0, 23_000.0);
    let mut failures = Vec::new();
    let (mut worst_passband, mut worst_elsewhere) = (0.0_f64, 0.0_f64);
    for case in tfar_designs() {
        let original = OracleFilter::new(case.original, SAMPLE_RATE_HZ)
            .unwrap()
            .magnitudes(&frequencies)
            .unwrap();
        let ours = case.ours.second_order_sections(SAMPLE_RATE_HZ).unwrap();
        for (frequency, original_gain) in frequencies.iter().zip(&original) {
            let our_gain = ours.magnitude_at(*frequency);
            let difference = decibel_difference(*original_gain, our_gain);
            let in_passband = *original_gain >= core::f64::consts::FRAC_1_SQRT_2;
            let tolerance = if in_passband {
                worst_passband = worst_passband.max(difference);
                PASSBAND_TOLERANCE_DB
            } else {
                worst_elsewhere = worst_elsewhere.max(difference);
                STOPBAND_TOLERANCE_DB
            };
            if difference.is_nan() || difference > tolerance {
                failures.push(format!(
                    "{} at {frequency:.1} Hz: DSPFilters {original_gain:e}, ours {our_gain:e}, \
                     {difference:.4} dB apart",
                    case.name
                ));
            }
        }
    }
    println!(
        "magnitude: worst {worst_passband:e} dB in the passband, {worst_elsewhere:e} dB elsewhere"
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Stereo `i16` noise within `±amplitude`, left and right independent.
fn stereo_i16_noise(frames: usize, amplitude: f64) -> Vec<i16> {
    let left = white_noise(frames, amplitude, 31);
    let right = white_noise(frames, amplitude, 32);
    left.iter()
        .zip(&right)
        .flat_map(|(left, right)| [*left as i16, *right as i16])
        .collect()
}

#[test]
fn the_stereo_i16_path_matches_process_filter_stereo_within_one_lsb_clipped_or_not() {
    let input = stereo_i16_noise(9_600, 20_000.0);
    let mut failures = Vec::new();
    let (mut worst_unclipped, mut worst_clipped) = (0_i32, 0_i32);
    let (mut differing_unclipped, mut differing_clipped) = (0_usize, 0_usize);
    for gain in [1.0_f32, 4.0] {
        for case in tfar_designs() {
            let mut original = input.clone();
            OracleFilter::new(case.original, SAMPLE_RATE_HZ)
                .unwrap()
                .process_stereo_i16(&mut original, gain)
                .unwrap();
            let mut buffer = StereoBuffer::with_frames(input.len() / 2);
            buffer.as_mut_slice().copy_from_slice(&input);
            StereoFilter::new(case.ours, SAMPLE_RATE_HZ)
                .unwrap()
                .process_stereo_i16(&mut buffer, gain);
            for (index, (expected, actual)) in original.iter().zip(buffer.as_slice()).enumerate() {
                let difference = (i32::from(*expected) - i32::from(*actual)).abs();
                let is_clipped = *expected == i16::MAX || *expected == i16::MIN;
                if difference > 0 {
                    if is_clipped {
                        differing_clipped += 1;
                    } else {
                        differing_unclipped += 1;
                    }
                }
                if is_clipped {
                    worst_clipped = worst_clipped.max(difference);
                } else {
                    worst_unclipped = worst_unclipped.max(difference);
                }
                let tolerance = 1;
                if difference > tolerance {
                    failures.push(format!(
                        "{} gain {gain} sample {index}: original {expected}, ours {actual}",
                        case.name
                    ));
                }
            }
        }
    }
    println!(
        "stereo i16: worst {worst_unclipped} LSB over {differing_unclipped} unclipped samples, \
         {worst_clipped} LSB over {differing_clipped} clipped samples"
    );
    failures.truncate(20);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_comparison_measures_see_a_shifted_cutoff() {
    // The check itself: a design one percent off must fail the RMS tolerance it is held to.
    let signal = white_noise(48_000, 0.5, 11);
    let mut original = to_f32(&signal);
    OracleFilter::new(
        OracleFilterDesign::ButterworthLowPass {
            order: 2,
            cutoff_hz: 5_000.0,
        },
        SAMPLE_RATE_HZ,
    )
    .unwrap()
    .process_mono_f32(&mut original)
    .unwrap();
    let mut ours: Vec<[f32; 1]> = to_f32(&signal).into_iter().map(|sample| [sample]).collect();
    MonoFilter::new(
        FilterDesign::ButterworthLowPass {
            order: 2,
            cutoff_hz: 5_050.0,
        },
        SAMPLE_RATE_HZ,
    )
    .unwrap()
    .process_frames_f32(&mut ours);
    let error = relative_rms_error(&to_f64(&original), &to_f64(ours.as_flattened())).unwrap();
    assert!(error > RELATIVE_RMS_TOLERANCE, "{error:e}");
    let peak = max_absolute_difference(&to_f64(&original), &to_f64(ours.as_flattened())).unwrap();
    assert!(peak > 0.0);
}
