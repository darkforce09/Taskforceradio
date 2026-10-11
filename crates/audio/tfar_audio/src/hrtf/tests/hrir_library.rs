//! Unit tests for `HrirLibrary`: the shipped sphere loads and resamples, the sphere's axes are the
//! ones the direction conversion assumes, and a missing or malformed file is an `Error`.

use super::*;
use crate::hrtf::real_sphere::{real_library, real_sphere_path};

/// A file path under the repository's `target/tmp/`, unique to this process and `name`.
fn temporary_path(name: &str) -> PathBuf {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../target/tmp");
    std::fs::create_dir_all(&folder).unwrap();
    folder.join(format!("tfar_hrtf_{}_{name}", std::process::id()))
}

#[test]
fn the_shipped_sphere_loads_and_is_resampled_to_48_khz() {
    let library = real_library();
    assert_eq!(library.point_count(), 187);
    assert_eq!(library.recorded_sample_rate_hz(), 44_100);
    // 512 samples at 44.1 kHz cover 11.6 ms, which is 558 samples at 48 kHz.
    assert_eq!(library.hrir_length(), 558);
    assert_eq!(library.source_path(), real_sphere_path());
}

#[test]
fn a_missing_file_is_an_unreadable_error() {
    let path = temporary_path("missing_sphere.bin");
    let error = HrirLibrary::load(&path).unwrap_err();
    assert!(
        matches!(&error, Error::SphereFileUnreadable { path: reported, .. } if *reported == path),
        "{error:?}"
    );
}

#[test]
fn a_malformed_file_is_refused_before_the_hrtf_crate_sees_it() {
    let path = temporary_path("malformed_sphere.bin");
    let mut bytes = std::fs::read(real_sphere_path()).unwrap();
    // A sample rate of zero would make the hrtf crate's resampler panic.
    bytes[4..8].copy_from_slice(&0_u32.to_le_bytes());
    std::fs::write(&path, &bytes).unwrap();
    let result = HrirLibrary::load(&path);
    std::fs::remove_file(&path).unwrap();
    assert!(
        matches!(result, Err(Error::SphereFileMalformed { .. })),
        "{result:?}"
    );
}

#[test]
fn the_sphere_is_y_up_measured_from_minus_45_to_plus_90_degrees() {
    let sphere = HrirSphere::from_file(real_sphere_path(), SAMPLE_RATE_HZ).unwrap();
    let heights = sphere.points().iter().map(|point| point.pos.y);
    let lowest = heights.clone().fold(f32::INFINITY, f32::min);
    let highest = heights.fold(f32::NEG_INFINITY, f32::max);
    assert!(
        (lowest - (-45.0_f32).to_radians().sin()).abs() < 1e-3,
        "{lowest}"
    );
    assert!((highest - 1.0).abs() < 1e-3, "{highest}");
}

#[test]
fn the_sphere_puts_the_right_ear_at_plus_x() {
    let sphere = HrirSphere::from_file(real_sphere_path(), SAMPLE_RATE_HZ).unwrap();
    let energy = |samples: &[f32]| samples.iter().map(|sample| sample * sample).sum::<f32>();
    let most_right_dominant = sphere
        .points()
        .iter()
        .max_by(|first, second| {
            let ratio =
                |point: &hrtf::HrirPoint| energy(point.right_hrir()) / energy(point.left_hrir());
            ratio(first).total_cmp(&ratio(second))
        })
        .unwrap();
    assert!(
        most_right_dominant.pos.x > 0.9,
        "{:?}",
        most_right_dominant.pos
    );
}
