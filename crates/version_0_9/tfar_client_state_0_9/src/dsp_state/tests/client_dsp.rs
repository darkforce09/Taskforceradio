//! Unit tests for `ClientDsp`, against the lazy getters and the two resets of `CLIENT_DATA`
//! (`reference/tfar_0_9_12/ts/src/plugin.cpp:232-341`).

use std::path::Path;
use std::sync::Arc;

use super::*;
use crate::dsp_state::{
    DirectVoiceRequirement, RadioEffectKind, RadioRequirement, SpeakerRequirement,
};

fn sphere() -> Arc<HrirLibrary> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../assets/hrir/IRC_1002_C.bin");
    HrirLibrary::load(path).unwrap()
}

fn radio(
    radio_id: &str,
    effect: RadioEffectKind,
    speaker: Option<SpeakerRequirement>,
) -> RadioRequirement {
    RadioRequirement {
        radio_id: radio_id.to_owned(),
        effect,
        speaker,
    }
}

fn everything() -> DspRequirements {
    DspRequirements {
        direct_voice: Some(DirectVoiceRequirement {
            vehicle_isolation: Some(0.5),
            needs_cant_speak_filter: true,
        }),
        radios: vec![
            radio("local_radio", RadioEffectKind::Personal, None),
            radio(
                "speaker_1",
                RadioEffectKind::LongRange,
                Some(SpeakerRequirement {
                    vehicle_isolation: Some(0.3),
                    is_under_water: true,
                }),
            ),
        ],
    }
}

#[test]
fn nothing_exists_until_it_is_ensured() {
    let mut dsp = ClientDsp::default();
    assert!(dsp.compressor_mut().is_none());
    assert!(dsp.holds_no_voice_state());
    assert!(dsp.radio_mut("local_radio").is_none());
}

#[test]
fn ensure_makes_exactly_what_the_requirements_name() {
    let mut dsp = ClientDsp::default();
    dsp.ensure(&everything(), None).unwrap();
    assert!(dsp.compressor_mut().is_some());
    let voice = dsp.direct_voice_mut();
    assert!(voice.cant_speak_filter_mut().is_some());
    assert!((voice.vehicle_filter_mut().unwrap().isolation() - 0.5).abs() < 1e-12);
    assert!(
        voice.hrtf_source_mut().is_none(),
        "no sphere, no direction source"
    );
    let local = dsp.radio_mut("local_radio").unwrap();
    assert!(local.personal_effect_mut().is_some());
    assert!(local.long_range_effect_mut().is_none());
    assert!(local.speaker_filter_mut().is_none());
    let speaker = dsp.radio_mut("speaker_1").unwrap();
    assert!(speaker.long_range_effect_mut().is_some());
    assert!(speaker.speaker_filter_mut().is_some());
    assert!(speaker.under_water_filter_mut().is_some());
    assert!((speaker.vehicle_filter_mut().unwrap().isolation() - 0.3).abs() < 1e-12);
    assert_eq!(
        dsp.radio_ids().collect::<Vec<_>>(),
        ["local_radio", "speaker_1"]
    );
}

#[test]
fn each_subtype_effect_is_made_in_its_own_slot() {
    let mut dsp = ClientDsp::default();
    let requirements = DspRequirements {
        direct_voice: None,
        radios: vec![
            radio("a", RadioEffectKind::UnderWater, None),
            radio("b", RadioEffectKind::Phone, None),
            radio("c", RadioEffectKind::Silent, None),
        ],
    };
    dsp.ensure(&requirements, None).unwrap();
    assert!(
        dsp.radio_mut("a")
            .unwrap()
            .under_water_effect_mut()
            .is_some()
    );
    assert!(dsp.radio_mut("b").unwrap().phone_filter_mut().is_some());
    assert!(dsp.radio_mut("c").unwrap().is_empty());
}

#[test]
fn a_direction_source_is_made_per_key_when_a_sphere_is_given() {
    let library = sphere();
    let mut dsp = ClientDsp::default();
    dsp.ensure(&everything(), Some(&library)).unwrap();
    assert!(dsp.direct_voice_mut().hrtf_source_mut().is_some());
    assert!(
        dsp.radio_mut("speaker_1")
            .unwrap()
            .hrtf_source_mut()
            .is_some()
    );
    assert!(
        dsp.radio_mut("local_radio")
            .unwrap()
            .hrtf_source_mut()
            .is_none(),
        "the local player's own radios play without a direction, as without a clunk"
    );
}

#[test]
fn ensure_keeps_existing_objects_and_their_memory() {
    let mut dsp = ClientDsp::default();
    dsp.ensure(&everything(), None).unwrap();
    let filter = dsp.direct_voice_mut().cant_speak_filter_mut().unwrap();
    let primed = filter.process_frame([0.5, 0.5]);
    dsp.ensure(&everything(), None).unwrap();
    let filter = dsp.direct_voice_mut().cant_speak_filter_mut().unwrap();
    let next = filter.process_frame([0.5, 0.5]);
    assert_ne!(
        primed, next,
        "a rebuilt filter would repeat its first output"
    );
}

#[test]
fn a_vehicle_filter_is_redesigned_only_for_a_new_isolation() {
    let mut dsp = ClientDsp::default();
    dsp.ensure(&everything(), None).unwrap();
    let mut changed = everything();
    if let Some(voice) = changed.direct_voice.as_mut() {
        voice.vehicle_isolation = Some(0.8);
    }
    dsp.ensure(&changed, None).unwrap();
    let isolation = dsp
        .direct_voice_mut()
        .vehicle_filter_mut()
        .unwrap()
        .isolation();
    assert!((isolation - 0.8).abs() < 1e-12);
}

#[test]
fn the_radio_effect_reset_drops_effects_and_speaker_filters_and_keeps_voices() {
    let mut dsp = ClientDsp::default();
    dsp.ensure(&everything(), None).unwrap();
    dsp.reset_radio_effects();
    assert!(
        dsp.radio_mut("local_radio").is_none(),
        "an emptied radio entry is removed"
    );
    let speaker = dsp.radio_mut("speaker_1").unwrap();
    assert!(speaker.long_range_effect_mut().is_none());
    assert!(speaker.speaker_filter_mut().is_none());
    assert!(speaker.under_water_filter_mut().is_some());
    assert!(speaker.vehicle_filter_mut().is_some());
    assert!(dsp.direct_voice_mut().cant_speak_filter_mut().is_some());
    assert!(dsp.compressor_mut().is_some());
}

#[test]
fn the_voice_reset_drops_directions_and_low_passes_and_keeps_effects() {
    let library = sphere();
    let mut dsp = ClientDsp::default();
    dsp.ensure(&everything(), Some(&library)).unwrap();
    dsp.reset_voices();
    assert!(dsp.direct_voice_mut().is_empty());
    let speaker = dsp.radio_mut("speaker_1").unwrap();
    assert!(speaker.hrtf_source_mut().is_none());
    assert!(speaker.under_water_filter_mut().is_none());
    assert!(speaker.vehicle_filter_mut().is_none());
    assert!(speaker.long_range_effect_mut().is_some());
    assert!(speaker.speaker_filter_mut().is_some());
    assert!(
        dsp.radio_mut("local_radio")
            .unwrap()
            .personal_effect_mut()
            .is_some()
    );
    assert!(
        dsp.compressor_mut().is_some(),
        "the compressor is never reset"
    );
}

#[test]
fn successive_dropout_seeds_differ() {
    assert_ne!(next_seed(FIRST_DROPOUT_SEED), FIRST_DROPOUT_SEED);
    assert_ne!(
        next_seed(next_seed(FIRST_DROPOUT_SEED)),
        next_seed(FIRST_DROPOUT_SEED)
    );
}
