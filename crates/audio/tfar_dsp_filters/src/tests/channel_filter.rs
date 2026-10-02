//! Unit tests for `ChannelFilter`: stability, measured gains, channel independence, the `sci-rs`
//! `sosfilt` cross-check, reset, the `f32` and mono paths, and the stereo `i16` path with clipping.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::float_cmp,
    reason = "the tests narrow samples on purpose and compare identical computations bit for bit"
)]

use super::*;
use sci_rs::signal::filter::design::Sos;
use sci_rs::signal::filter::sosfilt_dyn;

const SAMPLE_RATE_HZ: f64 = 48_000.0;

fn tfar_designs() -> [FilterDesign; 10] {
    [
        FilterDesign::ButterworthLowPass {
            order: 2,
            cutoff_hz: 5_000.0,
        },
        FilterDesign::ButterworthLowPass {
            order: 2,
            cutoff_hz: 2_000.0,
        },
        FilterDesign::ButterworthLowPass {
            order: 4,
            cutoff_hz: 100.0,
        },
        FilterDesign::ButterworthBandPass {
            order: 1,
            centre_hz: 2_000.0,
            width_hz: 1_000.0,
        },
        FilterDesign::ButterworthBandPass {
            order: 2,
            centre_hz: 1_850.0,
            width_hz: 1_550.0,
        },
        FilterDesign::ButterworthBandPass {
            order: 2,
            centre_hz: 1_000.0,
            width_hz: 400.0,
        },
        FilterDesign::RbjHighPass {
            cutoff_hz: 520.0,
            quality: 0.97,
        },
        FilterDesign::RbjLowPass {
            cutoff_hz: 1_300.0,
            quality: 1.0,
        },
        FilterDesign::RbjHighPass {
            cutoff_hz: 900.0,
            quality: 0.85,
        },
        FilterDesign::RbjLowPass {
            cutoff_hz: 3_000.0,
            quality: 2.0,
        },
    ]
}

/// A deterministic test signal: a few tones plus a linear-congruential noise in `[-0.5, 0.5]`.
fn test_signal(length: usize, seed: u64) -> Vec<f64> {
    let mut state = seed;
    (0..length)
        .map(|index| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let noise =
                f64::from(u32::try_from(state >> 40).unwrap()) / f64::from(1_u32 << 24) - 0.5;
            let time = index as f64 / SAMPLE_RATE_HZ;
            0.3 * (core::f64::consts::TAU * 440.0 * time).sin()
                + 0.2 * (core::f64::consts::TAU * 3_100.0 * time).sin()
                + noise
        })
        .collect()
}

/// The amplitude of the steady-state response to a unit sine at `frequency_hz` (a whole number
/// of hertz): `sqrt(2 * mean(y²))` over one second, a whole number of periods, after one second of
/// settling. Unlike the sample peak, this does not depend on where the samples fall in a period.
fn measured_gain(design: FilterDesign, frequency_hz: f64) -> f64 {
    let mut filter = MonoFilter::new(design, SAMPLE_RATE_HZ).unwrap();
    let settle: u32 = 48_000;
    let measured: u32 = 48_000;
    let energy: f64 = (0..settle + measured)
        .map(|index| {
            let time = f64::from(index) / SAMPLE_RATE_HZ;
            filter.process_sample((core::f64::consts::TAU * frequency_hz * time).sin())
        })
        .skip(usize::try_from(settle).unwrap())
        .map(|value| value * value)
        .sum();
    (2.0 * energy / f64::from(measured)).sqrt()
}

#[test]
fn every_tfar_design_has_a_decaying_impulse_response() {
    for design in tfar_designs() {
        let mut filter = MonoFilter::new(design, SAMPLE_RATE_HZ).unwrap();
        let response: Vec<f64> = (0..48_000)
            .map(|index| filter.process_sample(if index == 0 { 1.0 } else { 0.0 }))
            .collect();
        assert!(response.iter().all(|value| value.is_finite()), "{design:?}");
        let early: f64 = response[..4_800].iter().map(|value| value.abs()).sum();
        let tail: f64 = response[44_000..].iter().map(|value| value.abs()).sum();
        assert!(early > 1e-3, "{design:?} has no response");
        assert!(tail < 1e-9, "{design:?} does not decay: tail {tail}");
    }
}

#[test]
fn the_measured_gain_through_processing_matches_the_designed_magnitude() {
    for design in tfar_designs() {
        let sections = design.second_order_sections(SAMPLE_RATE_HZ).unwrap();
        for frequency_hz in [
            50.0, 100.0, 520.0, 1_000.0, 1_500.0, 2_000.0, 2_625.0, 5_000.0, 12_000.0,
        ] {
            let expected = sections.magnitude_at(frequency_hz);
            let measured = measured_gain(design, frequency_hz);
            assert!(
                (measured - expected).abs() < 2e-3 * expected.max(1e-2),
                "{design:?} at {frequency_hz}: measured {measured}, designed {expected}"
            );
        }
    }
}

#[test]
fn the_measured_band_pass_gain_is_three_db_down_at_both_band_edges() {
    for (order, centre_hz, width_hz) in [
        (1, 2_000.0, 1_000.0),
        (2, 1_850.0, 1_550.0),
        (2, 1_000.0, 400.0),
    ] {
        let design = FilterDesign::ButterworthBandPass {
            order,
            centre_hz,
            width_hz,
        };
        let (lower_hz, upper_hz) = design.band_edges_hz().unwrap();
        for edge_hz in [lower_hz, upper_hz] {
            let gain_db = 20.0 * measured_gain(design, edge_hz).log10();
            assert!(
                (gain_db + 3.0103).abs() < 0.01,
                "{design:?} at {edge_hz}: {gain_db} dB"
            );
        }
    }
}

#[test]
fn channels_are_filtered_independently() {
    let design = FilterDesign::ButterworthBandPass {
        order: 2,
        centre_hz: 1_850.0,
        width_hz: 1_550.0,
    };
    let mut stereo = StereoFilter::new(design, SAMPLE_RATE_HZ).unwrap();
    let mut left_only = MonoFilter::new(design, SAMPLE_RATE_HZ).unwrap();
    let mut right_only = MonoFilter::new(design, SAMPLE_RATE_HZ).unwrap();
    let left = test_signal(4_800, 1);
    let right = test_signal(4_800, 2);
    for (l, r) in left.iter().zip(&right) {
        let [out_l, out_r] = stereo.process_frame([*l, *r]);
        assert_eq!(out_l, left_only.process_sample(*l));
        assert_eq!(out_r, right_only.process_sample(*r));
    }
}

#[test]
fn a_silent_channel_stays_silent_beside_a_loud_one() {
    let design = FilterDesign::ButterworthLowPass {
        order: 4,
        cutoff_hz: 100.0,
    };
    let mut filter = ChannelFilter::<3>::new(design, SAMPLE_RATE_HZ).unwrap();
    let mut frames: Vec<[f64; 3]> = test_signal(2_000, 3)
        .into_iter()
        .map(|v| [v, 0.0, -v])
        .collect();
    filter.process_frames_f64(&mut frames);
    assert!(frames.iter().all(|[_, silent, _]| *silent == 0.0));
    assert!(frames.iter().all(|[a, _, b]| *a == -*b));
    assert!(frames.iter().any(|[a, _, _]| *a != 0.0));
}

#[test]
fn processing_matches_sci_rs_sosfilt_on_the_same_sections() {
    for design in tfar_designs() {
        let mut filter = MonoFilter::new(design, SAMPLE_RATE_HZ).unwrap();
        let mut reference: Vec<Sos<f64>> = filter
            .sections()
            .sections()
            .iter()
            .map(|section| {
                Sos::new(
                    [section.b0, section.b1, section.b2],
                    [1.0, section.a1, section.a2],
                )
            })
            .collect();
        let input = test_signal(9_600, 4);
        let expected = sosfilt_dyn(input.iter(), &mut reference);
        for (sample, expected) in input.iter().zip(expected) {
            let actual = filter.process_sample(*sample);
            assert!(
                (actual - expected).abs() < 1e-12,
                "{design:?}: {actual} vs {expected}"
            );
        }
    }
}

#[test]
fn reset_makes_the_filter_behave_as_new() {
    let design = FilterDesign::RbjLowPass {
        cutoff_hz: 1_300.0,
        quality: 1.0,
    };
    let mut used = StereoFilter::new(design, SAMPLE_RATE_HZ).unwrap();
    let mut frames: Vec<[f64; 2]> = test_signal(1_000, 5)
        .into_iter()
        .map(|v| [v, v * 0.5])
        .collect();
    used.process_frames_f64(&mut frames);
    used.reset();
    let mut fresh = StereoFilter::new(design, SAMPLE_RATE_HZ).unwrap();
    for value in test_signal(1_000, 6) {
        assert_eq!(
            used.process_frame([value, -value]),
            fresh.process_frame([value, -value])
        );
    }
}

#[test]
fn the_f32_path_matches_the_f64_path_to_f32_precision() {
    let design = FilterDesign::ButterworthBandPass {
        order: 1,
        centre_hz: 2_000.0,
        width_hz: 1_000.0,
    };
    let mut wide = StereoFilter::new(design, SAMPLE_RATE_HZ).unwrap();
    let mut narrow = StereoFilter::new(design, SAMPLE_RATE_HZ).unwrap();
    let signal = test_signal(4_800, 7);
    let mut frames_f32: Vec<[f32; 2]> = signal.iter().map(|v| [*v as f32, -*v as f32]).collect();
    let mut frames_f64: Vec<[f64; 2]> = frames_f32
        .iter()
        .map(|[l, r]| [f64::from(*l), f64::from(*r)])
        .collect();
    narrow.process_frames_f32(&mut frames_f32);
    wide.process_frames_f64(&mut frames_f64);
    for ([nl, nr], [wl, wr]) in frames_f32.iter().zip(&frames_f64) {
        assert_eq!(*nl, *wl as f32);
        assert_eq!(*nr, *wr as f32);
    }
}

/// The original `processFilterStereo` for one stereo block: split to `f32` by `/ 32767`, filter,
/// `* gain`, then above 1.0 `i16::MAX`, below -1.0 `i16::MIN`, else `* 32766` and truncate.
fn reference_stereo_i16(filter: &mut StereoFilter, samples: &[i16], gain: f32) -> Vec<i16> {
    samples
        .as_chunks::<2>()
        .0
        .iter()
        .flat_map(|frame| {
            let input = [
                f64::from(f32::from(frame[0]) / 32_767.0),
                f64::from(f32::from(frame[1]) / 32_767.0),
            ];
            filter.process_frame(input).map(|value| {
                let scaled = (value * f64::from(gain)) as f32;
                if scaled > 1.0 {
                    i16::MAX
                } else if scaled < -1.0 {
                    i16::MIN
                } else {
                    (scaled * 32_766.0) as i16
                }
            })
        })
        .collect()
}

fn stereo_block(samples: &[i16]) -> StereoBuffer {
    let mut buffer = StereoBuffer::with_frames(samples.len() / 2);
    buffer.as_mut_slice().copy_from_slice(samples);
    buffer
}

#[test]
fn the_stereo_i16_path_follows_process_filter_stereo() {
    let design = FilterDesign::ButterworthBandPass {
        order: 1,
        centre_hz: 2_000.0,
        width_hz: 1_000.0,
    };
    let mut filter = StereoFilter::new(design, SAMPLE_RATE_HZ).unwrap();
    let mut reference = StereoFilter::new(design, SAMPLE_RATE_HZ).unwrap();
    let samples: Vec<i16> = test_signal(1_920, 8)
        .iter()
        .map(|v| (v * 20_000.0) as i16)
        .collect();
    for gain in [1.0_f32, 4.0, 0.35] {
        let mut buffer = stereo_block(&samples);
        filter.process_stereo_i16(&mut buffer, gain);
        assert_eq!(
            buffer.as_slice(),
            reference_stereo_i16(&mut reference, &samples, gain).as_slice()
        );
    }
}

#[test]
fn a_low_pass_passes_steady_dc_through_the_i16_path() {
    let mut filter = StereoFilter::new(
        FilterDesign::ButterworthLowPass {
            order: 2,
            cutoff_hz: 5_000.0,
        },
        SAMPLE_RATE_HZ,
    )
    .unwrap();
    let mut buffer = stereo_block(&[16_384, -8_192].repeat(4_800));
    filter.process_stereo_i16(&mut buffer, 1.0);
    let last = &buffer.as_slice()[buffer.as_slice().len() - 2..];
    // 16384 / 32767 * 32766 = 16383.5 and -8192 / 32767 * 32766 = -8191.75, truncated.
    assert_eq!(last, &[16_383, -8_191]);
}

#[test]
fn the_stereo_i16_path_clips_to_i16_max_and_min_instead_of_wrapping() {
    let mut filter = StereoFilter::new(
        FilterDesign::ButterworthLowPass {
            order: 2,
            cutoff_hz: 5_000.0,
        },
        SAMPLE_RATE_HZ,
    )
    .unwrap();
    let mut buffer = stereo_block(&[16_384, -16_384].repeat(4_800));
    filter.process_stereo_i16(&mut buffer, 4.0);
    let last = &buffer.as_slice()[buffer.as_slice().len() - 2..];
    assert_eq!(last, &[i16::MAX, i16::MIN]);
}

#[test]
fn a_sample_beyond_full_scale_writes_i16_max_or_min_like_process_filter_stereo() {
    // helpers.hpp:185-187: above 1.0 is SHRT_MAX, below -1.0 SHRT_MIN, else `* (SHRT_MAX - 1)`.
    assert_eq!(clip_to_sample(1.000_001), i16::MAX);
    assert_eq!(clip_to_sample(-1.000_001), i16::MIN);
    assert_eq!(clip_to_sample(f32::INFINITY), i16::MAX);
    assert_eq!(clip_to_sample(f32::NEG_INFINITY), i16::MIN);
    assert_eq!(clip_to_sample(1.0), 32_766);
    assert_eq!(clip_to_sample(-1.0), -32_766);
    assert_eq!(clip_to_sample(0.5), 16_383);
    assert_eq!(clip_to_sample(f32::NAN), 0);
}

#[test]
fn a_nan_gain_silences_the_stereo_i16_block() {
    let mut filter = StereoFilter::new(
        FilterDesign::RbjLowPass {
            cutoff_hz: 3_000.0,
            quality: 2.0,
        },
        SAMPLE_RATE_HZ,
    )
    .unwrap();
    let mut buffer = stereo_block(&[12_000, -12_000].repeat(480));
    filter.process_stereo_i16(&mut buffer, f32::NAN);
    assert!(buffer.as_slice().iter().all(|sample| *sample == 0));
}

#[test]
fn an_empty_stereo_block_is_left_alone() {
    let mut filter = StereoFilter::new(
        FilterDesign::RbjHighPass {
            cutoff_hz: 520.0,
            quality: 0.97,
        },
        SAMPLE_RATE_HZ,
    )
    .unwrap();
    let mut buffer = StereoBuffer::with_frames(0);
    filter.process_stereo_i16(&mut buffer, 1.0);
    assert_eq!(buffer.frames(), 0);
}

#[test]
fn a_filter_built_from_sections_equals_one_built_from_the_design() {
    let design = FilterDesign::ButterworthLowPass {
        order: 4,
        cutoff_hz: 100.0,
    };
    let mut from_design = MonoFilter::new(design, SAMPLE_RATE_HZ).unwrap();
    let mut from_sections =
        MonoFilter::from_sections(design.second_order_sections(SAMPLE_RATE_HZ).unwrap());
    for value in test_signal(2_000, 9) {
        assert_eq!(
            from_design.process_sample(value),
            from_sections.process_sample(value)
        );
    }
}
