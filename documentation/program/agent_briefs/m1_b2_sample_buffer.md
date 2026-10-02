**Status:** live

# M1 · B2 · `tfar_sample_buffer`

You are agent B2, the audio-buffer author for the TFAR Rust port. Read
`/documentation/program/agent_briefs/shared_brief.md` first, then `/documentation/standards/coding_standards.md`
(including the TeamSpeak audio-thread rules).

## Goal
The interleaved 16-bit sample helpers every audio crate shares. They are used from TeamSpeak audio callbacks,
so no allocation happens in the per-frame paths, except in the functions explicitly named "owned".

## API (crates/audio/tfar_sample_buffer)
- **`StereoBuffer`:** an owned interleaved `Vec<i16>` with exactly 2 channels, and a frame count.
  - `with_frames(n)`, `clear`, `frames()`, `as_slice`, `as_mut_slice`, `copy_from(&StereoBuffer)`.
  - `reserve` up front, so no reallocation in steady state.
- **Gain:**
  - `apply_gain(&mut [i16], gain: f32)`. Gain 0 zeroes, gain 1 is a no-op, otherwise multiply with **saturation** to the i16 range (F-004; the wrapping original is at `reference/tfar_1_0/ts/src/SampleBuffer.cpp:36`).
  - `apply_stereo_gain(&mut StereoBuffer, left, right)`.
- **`mix_into_additive(dest, src)`:** saturating add, element-wise over the shorter length.
- **Channel views** for the TeamSpeak callbacks (F-006: the original read past the end for mono at `reference/tfar_0_9_12/ts/src/plugin.cpp:2831`):
  - `copy_to_stereo(samples: &[i16], channels: usize, frames: usize, out: &mut StereoBuffer) -> Result<()>`. Mono is duplicated, ≥2 channels take channels 0 and 1, 0 channels or a short slice is an `Error`.
  - `write_back_from_stereo(stereo, samples: &mut [i16], channels, frames)`:
    - mono gets the average of L and R (document why);
    - ≥2 channels write channels 0–1 and zero the channels ≥2, matching the original's zeroing;
    - bounds-checked.
  - `mono_to_stereo(mono: &[i16], out)`, for the captured-voice path.
- **Float conversion**, explicit constants per original convention:
  - Input: `i16 → f32` divides by `32767.0`. `INPUT_SCALE`.
  - Output: `f32 → i16` clips to [-1, 1], then multiplies by `32766.0`. `OUTPUT_SCALE`, as both originals' `processFilterStereo` and `processRadioEffect` do (`reference/tfar_1_0/ts/src/helpers.hpp:152`, `reference/tfar_1_0/ts/src/RadioEffect.hpp:230`).
  - Provide `to_float_planar(stereo, left: &mut [f32], right: &mut [f32])` and `from_float_planar`.
- **Tests:**
  - saturation at both ends;
  - gain 0 and 1 shortcuts;
  - mono / stereo / 6-channel views with exact expected arrays;
  - the short-slice error;
  - round-trip conversion error ≤ 1 LSB.

## Tracing setup (M1 checklist item)
Library crates only use `tracing` macros. They never install a subscriber; the plugin apps do that in M2
through `tfar_ts3_host`. Write this rule into `crates/audio/README.md` "How it works" only if it is missing, and say
whether you changed it. Don't add a tracing crate.

## Owned files
`crates/audio/tfar_sample_buffer/**`; `crates/audio/README.md` only for the tracing sentence.

## Checks (each alone, `CARGO_BUILD_JOBS=4`, logs to `target/agent-logs/b2-*.log`)
`cargo fmt -p tfar_sample_buffer`, `cargo clippy -p tfar_sample_buffer --all-targets -- -D warnings`,
`cargo test -p tfar_sample_buffer`; `cargo xtask verify all` if xtask exists (report foreign violations only).

## Report additions
Public API list, and the tests pinning F-004 and F-006.

Budget: S (150k tokens). Stop there and report what is done and what is not.
