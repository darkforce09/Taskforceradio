//! The structural check of an HRIR sphere file before the `hrtf` crate parses it.
//!
//! **Role:** reads the header of a sphere in mrDIMAS's `hrir_sphere_builder` format and refuses any
//! file whose sizes, indices or numbers would make the `hrtf` crate panic (and so abort TeamSpeak,
//! since the plugins build with `panic = "abort"`).
//!
//! **Position:** a module of `tfar_audio::hrtf`; called by `HrirLibrary::load` on the bytes of the file,
//! off the audio thread. The layout it checks is `HrtfSphere::Save` of the builder's
//! `src/hrtf_builder.cpp` and `HrirSphere::new` of the `hrtf` crate.
//!
//! **Signals and state:** none; a pure function over a byte slice.
//!
//! **Invariants:** a file passes only when it starts with `HRIR`, its sample rate lies in
//! 8 kHz..=192 kHz, its HRIR length in 1..=16384, it has at least four points and one triangle,
//! its size is exactly header + indices + points, every index names a point, and every coordinate
//! and impulse-response sample is finite, with no point at the origin.

#[cfg(test)]
#[path = "tests/sphere_file_check.rs"]
mod tests;

/// The bytes before the index buffer: magic, sample rate, HRIR length, point count, index count.
const HEADER_BYTES: usize = 20;
/// The magic the builder writes first.
const MAGIC: &[u8; 4] = b"HRIR";
/// The sample rates a sphere may be recorded at.
const SAMPLE_RATE_RANGE_HZ: core::ops::RangeInclusive<u32> = 8_000..=192_000;
/// The impulse-response lengths accepted, in samples (the IRCAM spheres use 512).
const HRIR_LENGTH_RANGE: core::ops::RangeInclusive<usize> = 1..=16_384;
/// The fewest points a closed triangulated sphere can have.
const MINIMUM_POINTS: usize = 4;
/// The bytes of one `u32` or `f32`.
const WORD_BYTES: usize = 4;

/// The header fields of a sphere file that passed the check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SphereFileLayout {
    /// The rate the impulse responses were recorded at, in Hz.
    pub(crate) sample_rate_hz: u32,
    /// The samples in each impulse response, per ear, before resampling.
    pub(crate) hrir_length: usize,
    /// The measured directions.
    pub(crate) point_count: usize,
    /// The triangles of the sphere mesh.
    pub(crate) triangle_count: usize,
}

/// Checks `bytes` as a whole sphere file and returns its layout, or the reason it is refused.
pub(crate) fn check_sphere_file(bytes: &[u8]) -> Result<SphereFileLayout, String> {
    if bytes.get(..MAGIC.len()) != Some(MAGIC.as_slice()) {
        return Err("the file does not start with the magic `HRIR`".to_owned());
    }
    let sample_rate_hz = read_word(bytes, 1)?;
    let hrir_length = word_as_count(read_word(bytes, 2)?)?;
    let point_count = word_as_count(read_word(bytes, 3)?)?;
    let index_count = word_as_count(read_word(bytes, 4)?)?;
    if !SAMPLE_RATE_RANGE_HZ.contains(&sample_rate_hz) {
        return Err(format!(
            "sample rate {sample_rate_hz} Hz lies outside {SAMPLE_RATE_RANGE_HZ:?}"
        ));
    }
    if !HRIR_LENGTH_RANGE.contains(&hrir_length) {
        return Err(format!(
            "HRIR length {hrir_length} lies outside {HRIR_LENGTH_RANGE:?}"
        ));
    }
    if point_count < MINIMUM_POINTS {
        return Err(format!(
            "{point_count} points; a sphere needs at least {MINIMUM_POINTS}"
        ));
    }
    if index_count == 0 || !index_count.is_multiple_of(3) {
        return Err(format!(
            "{index_count} indices; the mesh needs a positive multiple of three"
        ));
    }
    let point_bytes = hrir_length
        .checked_mul(2)
        .and_then(|samples| samples.checked_add(3))
        .and_then(|words| words.checked_mul(WORD_BYTES))
        .ok_or("the point size overflows")?;
    let expected_size = index_count
        .checked_mul(WORD_BYTES)
        .and_then(|index_bytes| index_bytes.checked_add(HEADER_BYTES))
        .and_then(|before_points| {
            point_count
                .checked_mul(point_bytes)
                .and_then(|all_points| before_points.checked_add(all_points))
        })
        .ok_or("the file size overflows")?;
    if bytes.len() != expected_size {
        return Err(format!(
            "the file holds {} bytes; its header describes {expected_size}",
            bytes.len()
        ));
    }
    let (index_bytes, point_records) = bytes
        .get(HEADER_BYTES..)
        .and_then(|rest| rest.split_at_checked(index_count * WORD_BYTES))
        .ok_or("the index buffer is cut short")?;
    check_indices(index_bytes, point_count)?;
    check_points(point_records, point_bytes)?;
    Ok(SphereFileLayout {
        sample_rate_hz,
        hrir_length,
        point_count,
        triangle_count: index_count / 3,
    })
}

/// Refuses an index that names no point.
fn check_indices(index_bytes: &[u8], point_count: usize) -> Result<(), String> {
    for (position, word) in index_bytes.as_chunks::<WORD_BYTES>().0.iter().enumerate() {
        let index = word_as_count(u32::from_le_bytes(*word))?;
        if index >= point_count {
            return Err(format!(
                "index {position} names point {index}, but there are {point_count} points"
            ));
        }
    }
    Ok(())
}

/// Refuses a point at the origin and any non-finite coordinate or sample.
fn check_points(point_records: &[u8], point_bytes: usize) -> Result<(), String> {
    for (point, record) in point_records.chunks_exact(point_bytes).enumerate() {
        let words = record.as_chunks::<WORD_BYTES>().0;
        let mut position_is_origin = true;
        for (word_index, word) in words.iter().enumerate() {
            let value = f32::from_le_bytes(*word);
            if !value.is_finite() {
                return Err(format!(
                    "point {point} holds a non-finite number at word {word_index}"
                ));
            }
            if word_index < 3 && value != 0.0 {
                position_is_origin = false;
            }
        }
        if position_is_origin {
            return Err(format!("point {point} lies at the origin"));
        }
    }
    Ok(())
}

/// The little-endian `u32` at word `word_index` of the header.
fn read_word(bytes: &[u8], word_index: usize) -> Result<u32, String> {
    let start = word_index * WORD_BYTES;
    bytes
        .get(start..start + WORD_BYTES)
        .and_then(|word| <[u8; WORD_BYTES]>::try_from(word).ok())
        .map(u32::from_le_bytes)
        .ok_or_else(|| "the header is cut short".to_owned())
}

/// A header count as `usize`.
fn word_as_count(word: u32) -> Result<usize, String> {
    usize::try_from(word).map_err(|_| format!("the count {word} does not fit in memory"))
}
