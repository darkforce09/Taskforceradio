//! Unit tests for `ClientEffects`.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use tfar_hrtf::HrirLibrary;

use super::*;
use crate::effects::object_interception_cutoff_hz;

fn library() -> Arc<HrirLibrary> {
    static LIBRARY: OnceLock<Arc<HrirLibrary>> = OnceLock::new();
    LIBRARY
        .get_or_init(|| {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../assets/hrir/IRC_1002_C.bin");
            HrirLibrary::load(path).unwrap()
        })
        .clone()
}

fn context(strength: f32) -> DspContext<'static> {
    DspContext {
        hrir_library: None,
        object_interception_strength: strength,
        dropout_seed: 7,
        frames_per_block: 960,
    }
}

#[test]
fn nothing_is_lent_before_it_is_ensured() {
    let mut effects = ClientEffects::new();
    assert!(effects.personal_radio_mut("local_radio").is_none());
    assert!(effects.direct_speech_compressor_mut().is_none());
    assert!(effects.object_interception_filter_mut(1).is_none());
    assert!(effects.vehicle_filter_mut("local_vehicle", 0.5).is_none());
}

#[test]
fn ensure_creates_each_object_once_and_keeps_it() {
    let mut effects = ClientEffects::new();
    let requirements = [
        DspRequirement::PersonalRadio("local_radio"),
        DspRequirement::LongRangeRadio("intercom"),
        DspRequirement::AirborneRadio("local_radio"),
        DspRequirement::UnderWaterRadio("local_radio"),
        DspRequirement::CantSpeakFilter("local_cantspeak"),
        DspRequirement::VehicleFilter {
            key: "local_vehicle",
            vehicle_volume_loss: 0.5,
        },
        DspRequirement::SpeakerFilter("2:390"),
        DspRequirement::PhoneFilter("local_radio"),
        DspRequirement::ObjectInterceptionFilter { object_count: 2 },
    ];
    // Nine objects plus the compressor pair.
    assert_eq!(
        effects.ensure_dsp(&requirements, &context(400.0)).unwrap(),
        10
    );
    assert_eq!(
        effects.ensure_dsp(&requirements, &context(400.0)).unwrap(),
        0
    );
    assert!(effects.personal_radio_mut("local_radio").is_some());
    assert!(effects.long_range_radio_mut("intercom").is_some());
    assert!(effects.airborne_radio_mut("local_radio").is_some());
    assert!(effects.underwater_radio_mut("local_radio").is_some());
    assert!(effects.cant_speak_filter_mut("local_cantspeak").is_some());
    assert!(effects.vehicle_filter_mut("local_vehicle", 0.5).is_some());
    assert!(effects.vehicle_filter_mut("local_vehicle", 0.4).is_none());
    assert!(effects.speaker_filter_mut("2:390").is_some());
    assert!(effects.phone_filter_mut("local_radio").is_some());
    assert!(effects.object_interception_buffer_mut().capacity_frames() >= 960);
}

/// F-013: direct speech and radio each own a compressor, so running one leaves the other's
/// envelope at rest.
#[test]
fn direct_speech_and_radio_each_own_a_compressor() {
    let mut effects = ClientEffects::new();
    effects.ensure_dsp(&[], &context(400.0)).unwrap();
    let direct = effects.direct_speech_compressor_mut().unwrap();
    for _ in 0..480 {
        direct.process_frame(30_000.0, 30_000.0);
    }
    assert!(direct.over_threshold_db() > 0.0);
    let radio = effects.radio_compressor_mut().unwrap();
    assert!(radio.over_threshold_db().abs() < f64::EPSILON);
}

#[test]
fn an_hrtf_source_without_a_sphere_is_an_error_that_spares_the_rest() {
    let mut effects = ClientEffects::new();
    let error = effects
        .ensure_dsp(
            &[
                DspRequirement::Hrtf("voice_clunk"),
                DspRequirement::SpeakerFilter("2:390"),
            ],
            &context(400.0),
        )
        .unwrap_err();
    assert!(matches!(error, Error::NoHrirLibrary { .. }));
    assert!(effects.speaker_filter_mut("2:390").is_some());
    assert!(effects.hrtf_source_mut("voice_clunk").is_none());
}

#[test]
fn an_hrtf_source_is_built_from_the_sphere() {
    let library = library();
    let mut effects = ClientEffects::new();
    let context = DspContext {
        hrir_library: Some(&library),
        ..context(400.0)
    };
    effects
        .ensure_dsp(&[DspRequirement::Hrtf("voice_clunk")], &context)
        .unwrap();
    assert!(effects.hrtf_source_mut("voice_clunk").is_some());
}

/// A further original bug, fixed: the original keeps the last strength in one static shared by every client,
/// so only the first client to notice a change redesigns its filters; each client here
/// remembers the strength its filters were designed with.
#[test]
fn every_client_redesigns_its_object_filters_when_the_strength_changes() {
    let mut first = ClientEffects::new();
    let mut second = ClientEffects::new();
    let requirement = [DspRequirement::ObjectInterceptionFilter { object_count: 1 }];
    first.ensure_dsp(&requirement, &context(400.0)).unwrap();
    second.ensure_dsp(&requirement, &context(400.0)).unwrap();
    assert_eq!(first.ensure_dsp(&requirement, &context(300.0)).unwrap(), 1);
    assert_eq!(second.ensure_dsp(&requirement, &context(300.0)).unwrap(), 1);
    assert_eq!(first.object_interception_strength(), Some(300.0));
    assert_eq!(second.object_interception_strength(), Some(300.0));
}

/// A further original bug, fixed: five objects at the default strength give a 0 Hz cutoff, which the original
/// designs as a degenerate low-pass; here it is silence.
#[test]
fn an_object_cutoff_at_or_below_zero_is_silence() {
    assert!(object_interception_cutoff_hz(5, 400.0).abs() < f64::EPSILON);
    assert!((object_interception_cutoff_hz(9, 400.0)).abs() < f64::EPSILON);
    assert!((object_interception_cutoff_hz(-3, 400.0) - 2000.0).abs() < f64::EPSILON);
    let mut effects = ClientEffects::new();
    effects
        .ensure_dsp(
            &[
                DspRequirement::ObjectInterceptionFilter { object_count: 5 },
                DspRequirement::ObjectInterceptionFilter { object_count: 4 },
            ],
            &context(400.0),
        )
        .unwrap();
    assert!(matches!(
        effects.object_interception_filter_mut(5),
        Some(ObjectInterceptionFilter::Silence)
    ));
    assert!(matches!(
        effects.object_interception_filter_mut(300),
        Some(ObjectInterceptionFilter::Silence)
    ));
    assert!(matches!(
        effects.object_interception_filter_mut(4),
        Some(ObjectInterceptionFilter::Filter(_))
    ));
}

/// A further original bug, fixed: `/tfar rstflt` resets the airborne effects too.
#[test]
fn resetting_radio_effects_drops_the_airborne_effects_too() {
    let mut effects = ClientEffects::new();
    let requirements = [
        DspRequirement::PersonalRadio("a"),
        DspRequirement::AirborneRadio("a"),
        DspRequirement::SpeakerFilter("a"),
        DspRequirement::CantSpeakFilter("a"),
    ];
    effects.ensure_dsp(&requirements, &context(400.0)).unwrap();
    effects.reset_radio_effects();
    assert!(effects.personal_radio_mut("a").is_none());
    assert!(effects.airborne_radio_mut("a").is_none());
    assert!(effects.speaker_filter_mut("a").is_none());
    assert!(effects.cant_speak_filter_mut("a").is_some());
}

#[test]
fn resetting_voices_keeps_the_cant_speak_and_vehicle_filters_and_clears_their_memory() {
    let mut effects = ClientEffects::new();
    let requirements = [
        DspRequirement::CantSpeakFilter("a"),
        DspRequirement::VehicleFilter {
            key: "a",
            vehicle_volume_loss: 0.2,
        },
        DspRequirement::PersonalRadio("a"),
    ];
    effects.ensure_dsp(&requirements, &context(400.0)).unwrap();
    let fresh = effects
        .vehicle_filter_mut("a", 0.2)
        .unwrap()
        .process_frame([0.5, 0.5]);
    let _ = effects
        .vehicle_filter_mut("a", 0.2)
        .unwrap()
        .process_frame([0.5, 0.5]);
    effects.reset_voices();
    assert!(effects.cant_speak_filter_mut("a").is_some());
    assert_eq!(
        effects
            .vehicle_filter_mut("a", 0.2)
            .unwrap()
            .process_frame([0.5, 0.5]),
        fresh,
        "the filter starts from silence again"
    );
    assert!(effects.personal_radio_mut("a").is_some());
}
