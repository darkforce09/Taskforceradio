//! Unit tests for `WaveLibrary` in `tfar_audio::playback_mixer`.

use std::path::PathBuf;

use super::*;

/// A WAV file under the repository's `target/tmp/`, removed when dropped.
struct TemporaryWave {
    path: PathBuf,
}

impl TemporaryWave {
    /// Writes `samples` as a WAV file of `spec` under a name unique to this test.
    fn write_i16(name: &str, spec: hound::WavSpec, samples: &[i16]) -> Self {
        let path = temporary_path(name);
        let mut writer = hound::WavWriter::create(&path, spec).unwrap();
        for sample in samples {
            writer.write_sample(*sample).unwrap();
        }
        writer.finalize().unwrap();
        Self { path }
    }

    /// Writes `samples` as a 32-bit float WAV file under a name unique to this test.
    fn write_f32(name: &str, samples: &[f32]) -> Self {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let path = temporary_path(name);
        let mut writer = hound::WavWriter::create(&path, spec).unwrap();
        for sample in samples {
            writer.write_sample(*sample).unwrap();
        }
        writer.finalize().unwrap();
        Self { path }
    }
}

impl Drop for TemporaryWave {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// A file path under the repository's `target/tmp/`, unique to this process and `name`.
fn temporary_path(name: &str) -> PathBuf {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../target/tmp");
    std::fs::create_dir_all(&folder).unwrap();
    folder.join(format!(
        "tfar_playback_mixer_{}_{name}.wav",
        std::process::id()
    ))
}

fn spec(sample_rate: u32, channels: u16, bits_per_sample: u16) -> hound::WavSpec {
    hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample,
        sample_format: hound::SampleFormat::Int,
    }
}

#[test]
fn a_48_khz_16_bit_stereo_file_loads_as_interleaved_samples() {
    let wave = TemporaryWave::write_i16("valid", spec(48_000, 2, 16), &[1, -1, 300, -300]);
    let library = WaveLibrary::new();
    let samples = library.load(&wave.path).unwrap();
    assert_eq!(&*samples, &[1, -1, 300, -300]);
    assert_eq!(library.len(), 1);
}

#[test]
fn every_other_format_is_refused_with_the_file_and_its_format() {
    let library = WaveLibrary::new();
    let cases = [
        ("rate_44100", spec(44_100, 2, 16), 44_100, 2, 16),
        ("mono", spec(48_000, 1, 16), 48_000, 1, 16),
        ("eight_bit", spec(48_000, 2, 8), 48_000, 2, 8),
        ("twenty_four_bit", spec(48_000, 2, 24), 48_000, 2, 24),
    ];
    for (name, wave_spec, rate, channel_count, bits) in cases {
        let wave = if bits == 8 {
            let path = temporary_path(name);
            let mut writer = hound::WavWriter::create(&path, wave_spec).unwrap();
            writer.write_sample(1_i8).unwrap();
            writer.write_sample(1_i8).unwrap();
            writer.finalize().unwrap();
            TemporaryWave { path }
        } else {
            TemporaryWave::write_i16(name, wave_spec, &[1, 2])
        };
        let error = library.load(&wave.path).unwrap_err();
        match &error {
            Error::UnsupportedWaveFormat {
                path,
                sample_rate,
                channels,
                bits_per_sample,
                sample_format,
            } => {
                assert_eq!(path, &std::fs::canonicalize(&wave.path).unwrap());
                assert_eq!(*sample_rate, rate);
                assert_eq!(*channels, channel_count);
                assert_eq!(*bits_per_sample, bits);
                assert_eq!(*sample_format, hound::SampleFormat::Int);
            }
            other => panic!("{name}: expected a format error, got {other:?}"),
        }
        let message = error.to_string();
        assert!(message.contains(name), "{message}");
        assert!(message.contains(&format!("{rate} Hz")), "{message}");
    }
    assert!(library.is_empty(), "no refused file is cached");
}

#[test]
fn a_float_file_is_refused() {
    let wave = TemporaryWave::write_f32("float", &[0.5, -0.5]);
    let error = WaveLibrary::new().load(&wave.path).unwrap_err();
    assert!(matches!(
        error,
        Error::UnsupportedWaveFormat {
            sample_format: hound::SampleFormat::Float,
            bits_per_sample: 32,
            ..
        }
    ));
}

#[test]
fn a_missing_file_is_an_error_naming_it() {
    let path = temporary_path("missing");
    let error = WaveLibrary::new().load(&path).unwrap_err();
    assert!(matches!(&error, Error::WavePath { path: reported, .. } if *reported == path));
    assert!(error.to_string().contains("missing"));
}

#[test]
fn a_file_that_is_not_wav_is_a_read_error() {
    let path = temporary_path("not_wav");
    std::fs::write(&path, b"this is not a RIFF file").unwrap();
    let wave = TemporaryWave { path };
    assert!(matches!(
        WaveLibrary::new().load(&wave.path),
        Err(Error::WaveRead { .. })
    ));
}

#[test]
fn a_second_load_is_a_cache_hit_under_any_spelling_of_the_path() {
    let wave = TemporaryWave::write_i16("cache_hit", spec(48_000, 2, 16), &[5, 6, 7, 8]);
    let library = WaveLibrary::new();
    let first = library.load(&wave.path).unwrap();
    let parent = wave.path.parent().unwrap();
    let spelled_differently = parent.join(".").join(wave.path.file_name().unwrap());
    let second = library.load(&spelled_differently).unwrap();
    assert!(
        Arc::ptr_eq(&first, &second),
        "the same decoded samples are shared"
    );
    assert_eq!(library.len(), 1);
    // A hit does not read the file again: it still loads after the file is gone.
    std::fs::write(&wave.path, b"overwritten").unwrap();
    let third = library.load(&wave.path).unwrap();
    assert!(Arc::ptr_eq(&first, &third));
    assert!(library.cached(&wave.path).is_some());
}

#[test]
fn preload_caches_every_good_file_and_reports_each_bad_one() {
    let good = TemporaryWave::write_i16("preload_good", spec(48_000, 2, 16), &[1, 2]);
    let bad = TemporaryWave::write_i16("preload_bad", spec(22_050, 2, 16), &[1, 2]);
    let missing = temporary_path("preload_missing");
    let library = WaveLibrary::new();
    let errors = library.preload([&good.path, &bad.path, &missing]);
    assert_eq!(errors.len(), 2);
    assert!(matches!(errors[0], Error::UnsupportedWaveFormat { .. }));
    assert!(matches!(errors[1], Error::WavePath { .. }));
    assert!(library.cached(&good.path).is_some());
    assert_eq!(library.len(), 1);
}
