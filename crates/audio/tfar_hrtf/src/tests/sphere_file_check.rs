//! Unit tests for the sphere file check: the shipped sphere passes, and each kind of damage that
//! would make the `hrtf` crate panic is refused.

use super::*;
use crate::real_sphere::real_sphere_path;

/// A tetrahedron sphere in the builder's format, with impulse responses of `hrir_length` samples.
fn tetrahedron_sphere(hrir_length: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(b"HRIR");
    for word in [44_100_u32, hrir_length, 4, 12] {
        bytes.extend(word.to_le_bytes());
    }
    for index in [0_u32, 1, 2, 0, 3, 1, 0, 2, 3, 1, 3, 2] {
        bytes.extend(index.to_le_bytes());
    }
    for position in [
        [1.0_f32, 1.0, 1.0],
        [-1.0, -1.0, 1.0],
        [-1.0, 1.0, -1.0],
        [1.0, -1.0, -1.0],
    ] {
        for coordinate in position {
            bytes.extend(coordinate.to_le_bytes());
        }
        for sample in 0..2 * hrir_length {
            let value = if sample % hrir_length == 0 {
                1.0_f32
            } else {
                0.0
            };
            bytes.extend(value.to_le_bytes());
        }
    }
    bytes
}

/// Overwrites the little-endian word at byte `offset`.
fn put_word(bytes: &mut [u8], offset: usize, word: [u8; 4]) {
    bytes[offset..offset + 4].copy_from_slice(&word);
}

/// The byte offset of the first word of point `point` in a tetrahedron sphere.
fn point_offset(point: usize, hrir_length: usize) -> usize {
    20 + 12 * 4 + point * (12 + 8 * hrir_length)
}

#[test]
fn the_shipped_ircam_sphere_passes_with_its_layout() {
    let bytes = std::fs::read(real_sphere_path()).unwrap();
    let layout = check_sphere_file(&bytes).unwrap();
    assert_eq!(
        layout,
        SphereFileLayout {
            sample_rate_hz: 44_100,
            hrir_length: 512,
            point_count: 187,
            triangle_count: 370,
        }
    );
}

#[test]
fn a_well_formed_tetrahedron_passes() {
    assert!(check_sphere_file(&tetrahedron_sphere(8)).is_ok());
}

#[test]
fn a_wrong_magic_is_refused() {
    let mut bytes = tetrahedron_sphere(8);
    bytes[3] = b'X';
    assert!(check_sphere_file(&bytes).unwrap_err().contains("magic"));
}

#[test]
fn a_cut_header_is_refused() {
    assert!(check_sphere_file(b"HRIR\x44\xac").is_err());
}

#[test]
fn a_cut_or_padded_file_is_refused() {
    let bytes = tetrahedron_sphere(8);
    assert!(check_sphere_file(&bytes[..bytes.len() - 1]).is_err());
    let mut padded = bytes;
    padded.push(0);
    assert!(check_sphere_file(&padded).unwrap_err().contains("bytes"));
}

#[test]
fn a_zero_or_absurd_sample_rate_is_refused() {
    for rate in [0_u32, 7_999, 192_001] {
        let mut bytes = tetrahedron_sphere(8);
        put_word(&mut bytes, 4, rate.to_le_bytes());
        assert!(
            check_sphere_file(&bytes)
                .unwrap_err()
                .contains("sample rate"),
            "{rate} Hz"
        );
    }
}

#[test]
fn a_zero_hrir_length_is_refused() {
    let mut bytes = tetrahedron_sphere(8);
    put_word(&mut bytes, 8, 0_u32.to_le_bytes());
    assert!(
        check_sphere_file(&bytes)
            .unwrap_err()
            .contains("HRIR length")
    );
}

#[test]
fn too_few_points_are_refused() {
    let mut bytes = tetrahedron_sphere(8);
    put_word(&mut bytes, 12, 3_u32.to_le_bytes());
    assert!(check_sphere_file(&bytes).unwrap_err().contains("points"));
}

#[test]
fn an_index_count_that_is_no_multiple_of_three_is_refused() {
    let mut bytes = tetrahedron_sphere(8);
    put_word(&mut bytes, 16, 11_u32.to_le_bytes());
    assert!(check_sphere_file(&bytes).unwrap_err().contains("indices"));
}

#[test]
fn an_index_past_the_last_point_is_refused() {
    let mut bytes = tetrahedron_sphere(8);
    put_word(&mut bytes, 20 + 5 * 4, 4_u32.to_le_bytes());
    assert!(
        check_sphere_file(&bytes)
            .unwrap_err()
            .contains("names point 4")
    );
}

#[test]
fn a_non_finite_sample_is_refused() {
    let mut bytes = tetrahedron_sphere(8);
    put_word(
        &mut bytes,
        point_offset(2, 8) + 12 + 4,
        f32::NAN.to_le_bytes(),
    );
    assert!(
        check_sphere_file(&bytes)
            .unwrap_err()
            .contains("non-finite")
    );
}

#[test]
fn a_point_at_the_origin_is_refused() {
    let mut bytes = tetrahedron_sphere(8);
    for coordinate in 0..3 {
        put_word(
            &mut bytes,
            point_offset(1, 8) + coordinate * 4,
            0.0_f32.to_le_bytes(),
        );
    }
    assert!(check_sphere_file(&bytes).unwrap_err().contains("origin"));
}
