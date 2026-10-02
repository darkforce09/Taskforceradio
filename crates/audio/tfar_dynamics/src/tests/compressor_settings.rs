//! Unit tests for `CompressorSettings`: the presets and the range checks.

use super::*;

#[test]
fn the_0_9_preset_matches_the_0_9_plugin() {
    let preset = CompressorSettings::VERSION_0_9;
    assert_eq!(preset.sample_rate_hz, 48_000.0);
    assert_eq!(preset.threshold_db, 65.0);
    assert_eq!(preset.ratio, 0.1);
    assert_eq!(preset.attack_ms, 1.0);
    assert_eq!(preset.release_ms, 300.0);
}

#[test]
fn the_1_0_preset_differs_from_the_0_9_preset_only_in_its_threshold() {
    let preset = CompressorSettings::VERSION_1_0;
    assert_eq!(preset.threshold_db, 80.0);
    assert_eq!(
        CompressorSettings {
            threshold_db: 65.0,
            ..preset
        },
        CompressorSettings::VERSION_0_9
    );
}

#[test]
fn both_presets_are_valid() {
    assert!(CompressorSettings::VERSION_0_9.validate().is_ok());
    assert!(CompressorSettings::VERSION_1_0.validate().is_ok());
}

#[test]
fn every_setting_outside_its_range_is_named_in_the_error() {
    let base = CompressorSettings::VERSION_1_0;
    let cases = [
        (
            "sample_rate_hz",
            CompressorSettings {
                sample_rate_hz: 0.0,
                ..base
            },
        ),
        (
            "sample_rate_hz",
            CompressorSettings {
                sample_rate_hz: f64::NAN,
                ..base
            },
        ),
        (
            "threshold_db",
            CompressorSettings {
                threshold_db: f64::INFINITY,
                ..base
            },
        ),
        ("ratio", CompressorSettings { ratio: 0.0, ..base }),
        (
            "ratio",
            CompressorSettings {
                ratio: -0.1,
                ..base
            },
        ),
        (
            "attack_ms",
            CompressorSettings {
                attack_ms: 0.0,
                ..base
            },
        ),
        (
            "release_ms",
            CompressorSettings {
                release_ms: f64::NEG_INFINITY,
                ..base
            },
        ),
    ];
    for (expected, settings) in cases {
        match settings.validate() {
            Err(Error::InvalidSetting { setting, .. }) => assert_eq!(setting, expected),
            other => panic!("{settings:?}: {other:?}"),
        }
    }
}

#[test]
fn a_negative_threshold_is_valid() {
    let settings = CompressorSettings {
        threshold_db: -10.0,
        ..CompressorSettings::VERSION_1_0
    };
    assert!(settings.validate().is_ok());
}
