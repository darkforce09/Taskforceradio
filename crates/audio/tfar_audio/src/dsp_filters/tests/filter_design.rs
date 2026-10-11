//! Unit tests for `FilterDesign`: every design TFAR uses builds at 48 kHz, each variant reaches
//! its design, and bad parameters are errors.

use super::*;
use crate::dsp_filters::error::Error;

const SAMPLE_RATE_HZ: f64 = 48_000.0;

/// Every design the two original plugins set up (behaviour specs, §3).
fn tfar_designs() -> Vec<FilterDesign> {
    vec![
        // Vehicle muffling and object interception: LP2 at 20000 * (1 - loss) / 4 and
        // 2000 - n * strength.
        FilterDesign::ButterworthLowPass {
            order: 2,
            cutoff_hz: 5_000.0,
        },
        FilterDesign::ButterworthLowPass {
            order: 2,
            cutoff_hz: 2_000.0,
        },
        // Cannot speak and underwater: LP4 at 100 Hz.
        FilterDesign::ButterworthLowPass {
            order: 4,
            cutoff_hz: 100.0,
        },
        // Speaker: BP1 2000/1000.
        FilterDesign::ButterworthBandPass {
            order: 1,
            centre_hz: 2_000.0,
            width_hz: 1_000.0,
        },
        // Phone: BP2 1850/1550.
        FilterDesign::ButterworthBandPass {
            order: 2,
            centre_hz: 1_850.0,
            width_hz: 1_550.0,
        },
        // Underwater radio: BP2 1000/400.
        FilterDesign::ButterworthBandPass {
            order: 2,
            centre_hz: 1_000.0,
            width_hz: 400.0,
        },
        // Radio effects.
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

#[test]
fn every_tfar_design_builds_at_48_khz() {
    for design in tfar_designs() {
        let sections = design.second_order_sections(SAMPLE_RATE_HZ).unwrap();
        assert_eq!(sections.sample_rate_hz(), SAMPLE_RATE_HZ);
        assert_ne!(sections.sections(), []);
    }
}

#[test]
fn each_variant_reaches_its_own_design() {
    let low_pass = FilterDesign::ButterworthLowPass {
        order: 4,
        cutoff_hz: 100.0,
    };
    assert_eq!(
        low_pass
            .second_order_sections(SAMPLE_RATE_HZ)
            .unwrap()
            .sections()
            .len(),
        2
    );
    let band_pass = FilterDesign::ButterworthBandPass {
        order: 2,
        centre_hz: 1_000.0,
        width_hz: 400.0,
    };
    let band = band_pass.second_order_sections(SAMPLE_RATE_HZ).unwrap();
    assert_eq!(band.sections().len(), 2);
    assert!(band.magnitude_at(0.0) < 1e-9);
    let rbj_low = FilterDesign::RbjLowPass {
        cutoff_hz: 1_300.0,
        quality: 1.0,
    };
    assert!(
        (rbj_low
            .second_order_sections(SAMPLE_RATE_HZ)
            .unwrap()
            .magnitude_at(0.0)
            - 1.0)
            .abs()
            < 1e-12
    );
    let rbj_high = FilterDesign::RbjHighPass {
        cutoff_hz: 520.0,
        quality: 0.97,
    };
    assert!(
        rbj_high
            .second_order_sections(SAMPLE_RATE_HZ)
            .unwrap()
            .magnitude_at(0.0)
            < 1e-12
    );
}

#[test]
fn only_the_band_pass_has_band_edges() {
    let band_pass = FilterDesign::ButterworthBandPass {
        order: 2,
        centre_hz: 1_850.0,
        width_hz: 1_550.0,
    };
    assert_eq!(band_pass.band_edges_hz(), Some((1_075.0, 2_625.0)));
    assert_eq!(
        FilterDesign::ButterworthLowPass {
            order: 2,
            cutoff_hz: 1.0
        }
        .band_edges_hz(),
        None
    );
    assert_eq!(
        FilterDesign::RbjLowPass {
            cutoff_hz: 1.0,
            quality: 1.0
        }
        .band_edges_hz(),
        None
    );
    assert_eq!(
        FilterDesign::RbjHighPass {
            cutoff_hz: 1.0,
            quality: 1.0
        }
        .band_edges_hz(),
        None
    );
}

#[test]
fn bad_parameters_are_errors_for_every_variant() {
    let cases = [
        FilterDesign::ButterworthLowPass {
            order: 2,
            cutoff_hz: -5.0,
        },
        FilterDesign::ButterworthLowPass {
            order: 2,
            cutoff_hz: 24_000.0,
        },
        FilterDesign::ButterworthLowPass {
            order: 0,
            cutoff_hz: 100.0,
        },
        FilterDesign::ButterworthBandPass {
            order: 2,
            centre_hz: 1_000.0,
            width_hz: 0.0,
        },
        FilterDesign::ButterworthBandPass {
            order: 2,
            centre_hz: f64::NAN,
            width_hz: 400.0,
        },
        FilterDesign::RbjLowPass {
            cutoff_hz: f64::INFINITY,
            quality: 1.0,
        },
        FilterDesign::RbjHighPass {
            cutoff_hz: 520.0,
            quality: -0.97,
        },
    ];
    for design in cases {
        let result = design.second_order_sections(SAMPLE_RATE_HZ);
        assert!(
            matches!(
                result,
                Err(Error::FrequencyOutOfRange { .. }
                    | Error::InvalidOrder { .. }
                    | Error::InvalidBandWidth { .. }
                    | Error::InvalidQuality { .. })
            ),
            "{design:?}: {result:?}"
        );
    }
}

#[test]
fn a_bad_sample_rate_is_an_error_for_every_variant() {
    for design in tfar_designs() {
        assert!(matches!(
            design.second_order_sections(0.0),
            Err(Error::InvalidSampleRate { .. })
        ));
    }
}
