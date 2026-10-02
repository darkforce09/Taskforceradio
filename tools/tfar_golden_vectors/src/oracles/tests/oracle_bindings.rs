//! Unit tests for `oracle_bindings`: the shared struct layouts and the buffer checks.

use super::*;

#[test]
fn every_shared_struct_has_the_size_the_shim_compiles() {
    let sizes = struct_sizes();
    assert_eq!(size_of::<ShimFilterDesign>(), sizes.filter_design);
    assert_eq!(
        size_of::<ShimCompressorSettings>(),
        sizes.compressor_settings
    );
    assert_eq!(size_of::<ShimVector>(), sizes.vector);
    assert_eq!(size_of::<ShimSpatialScene>(), sizes.spatial_scene);
    assert_eq!(size_of::<ShimSpatialResult>(), sizes.spatial_result);
}

#[test]
fn an_odd_stereo_buffer_is_refused_before_the_shim_is_called() {
    let design = ShimFilterDesign {
        kind: KIND_RBJ_LOW_PASS,
        sample_rate_hz: 48_000.0,
        frequency_hz: 1_000.0,
        quality: 1.0,
        ..ShimFilterDesign::default()
    };
    let mut samples = [0_i16; 3];
    assert_eq!(
        filter_process_stereo_i16(&design, &mut samples, 1.0),
        Err(Error::UnevenFrames {
            samples: 3,
            channels: 2
        })
    );
}

#[test]
fn an_unknown_filter_kind_is_refused_by_the_shim() {
    let design = ShimFilterDesign {
        kind: 99,
        sample_rate_hz: 48_000.0,
        frequency_hz: 1_000.0,
        ..ShimFilterDesign::default()
    };
    let mut samples = [0.0_f32; 4];
    assert_eq!(
        filter_process_f32(&design, 1, &mut samples),
        Err(Error::OracleRefused {
            operation: "oracle_filter_process_f32",
            status: 1
        })
    );
}

#[test]
fn a_gain_buffer_of_the_wrong_length_is_refused() {
    let settings = ShimCompressorSettings {
        sample_rate_hz: 48_000.0,
        threshold_db: 80.0,
        ratio: 0.1,
        attack_ms: 1.0,
        release_ms: 300.0,
    };
    let mut samples = [0.0; 4];
    let mut gains = [0.0; 1];
    assert_eq!(
        simple_comp_process(&settings, &mut samples, &mut gains),
        Err(Error::LengthMismatch {
            reference: 2,
            candidate: 1
        })
    );
}

#[test]
fn a_scene_with_too_many_curve_points_is_refused_by_the_shim() {
    let scene = ShimSpatialScene {
        curve_point_count: 33,
        curve_distance_scaler: 1.0,
        ..ShimSpatialScene::default()
    };
    assert_eq!(
        f3daudio_calculate(&scene),
        Err(Error::OracleRefused {
            operation: "oracle_f3daudio_calculate",
            status: 1
        })
    );
}
