# Spatial audio

The `tfar_spatial_audio` crate: the distance curve and emitter cone gains that replace X3DAudio,
plus the simple stereo panner both versions fall back to. Used by `tfar_voice_pipeline_0_9`,
`tfar_voice_pipeline_1_0`.

## How it works

The 1.0 plugin got level and direction for direct speech from X3DAudio. Here `tfar_hrtf` gives
direction and this crate gives level. `DistanceCurve` is X3DAudio's volume curve: sorted
`(normalised distance, gain)` points, read by dividing the distance by the emitter's scaler and
interpolating linearly, holding the end values outside the points; `DistanceCurve::tfar_voice`
builds the original 25-point voice curve from the version's `volumeAttenuation`. `EmitterCone` is
X3DAudio's cone: inner volume within half the inner angle of the speaker's facing, outer volume
beyond half the outer angle, linear between; `EmitterCone::TFAR_VOICE` is the 1.0 voice cone. The
simple interaural level difference, `stereo_pan_gains`, gives `0.625 ∓ c·cos(dir)` with the 0.9 or
the 1.0 coefficient, wrapping `dir` into (-π, π], and `apply_pan` applies the pair. FAudio's
`F3DAudio.c` is the oracle the golden vectors compare the curve and cone against.

`lib.rs` declares the modules privately and re-exports their public items at the crate root;
`prelude.rs` re-exports the same items except `Error` and `Result`, so a glob import never
shadows `std`'s `Result`. `DistanceCurve` keeps its points in a fixed array of
`DistanceCurve::MAX_POINTS`, so it is `Copy` and never allocates; its constructor checks every
point and refuses unsorted ones. `EmitterCone` works on the `glam` vectors inside
`tfar_geometry::Direction`. Each module attaches its sibling test file in `tests/` with `#[path]`.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_spatial_audio  # the unit tests
```

## Configuration

None: the crate reads no setting. The 1.0 `voiceCone` setting decides in the voice pipeline
whether the cone applies.

## Public surface

- `DistanceCurve`: `new`, `tfar_voice`, `points`, `gain_at`, `MAX_POINTS`; `CurvePoint`.
- `EmitterCone`: `new`, `TFAR_VOICE`, `gain`, `inner_angle`, `outer_angle`, `inner_volume`,
  `outer_volume`.
- `stereo_pan_gains`, `apply_pan`, `ILD_COEFFICIENT_0_9`, `ILD_COEFFICIENT_1_0`, `ILD_BASE_GAIN`:
  the simple interaural level difference.
- `Error`, `Result` and `prelude`.

## Boundaries

- Depends on: `tfar_geometry` (directions and angles); `tfar_sample_buffer` (the stereo buffer
  and saturating stereo gain); `thiserror` for the error type.
- Used by: `tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0`.
- Rules: every gain is finite; nothing allocates, construction included, so curves and cones may
  be built on the TeamSpeak audio thread. Tier 1 in `crates/audio`: every workspace dependency has
  a lower tier and follows the category edge matrix (`cargo xtask verify crate-tiers`).

## Related documentation

- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — where the curve, the cone and
  the panner sit in each voice chain.
- [TFAR 0.9.12 plugin behaviour](/documentation/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
- [TFAR 1.0 plugin behaviour](/documentation/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
