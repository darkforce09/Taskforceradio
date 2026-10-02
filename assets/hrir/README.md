# HRIR sphere

The head-related impulse responses the binaural HRTF renders with: one IRCAM sphere in the `.bin`
format the `hrtf` crate reads, and CREDITS.md, its source, checksum, IRCAM credit and terms.

## Contents

```text
assets/hrir/
├── IRC_1002_C.bin  LISTEN subject 1002, compensated: 187 directions, 512 samples per ear, 44.1 kHz
└── CREDITS.md      the source URL and commit, the SHA-256, the IRCAM LISTEN credit and the terms
```

## Format

- Encoding: little-endian binary, the layout `HrtfSphere::Save` of mrDIMAS's
  hrir_sphere_builder writes: the magic `HRIR`, then `u32` sample rate (44100), HRIR length (512),
  point count (187) and index count (1110); 370 triangles as `u32` index triples; then per point
  its `f32` x, y, z followed by the left and the right impulse response as `f32` samples.
- Axes: right-handed, x to the listener's right, y up, z backwards; the frontal measurement lies
  at (0, 0, −1). The elevations run from −45° to +90°, so nothing is measured far below the head.
- Loading: `tfar_hrtf` checks the structure first, then the `hrtf` crate resamples every response
  to 48 kHz (558 samples) and prepares it for its overlap-save convolution.
- Why this subject: the `hrtf` crate's repository and package carry no example or test sphere
  (its tests build synthetic spheres), so the first subject of the base, 1002, is used; every
  compensated subject has the same layout, so another one can replace it with no code change.
- Adding a file: a new sphere comes with its entry in CREDITS.md (source, commit, SHA-256, terms)
  and passes the `tfar_hrtf` tests, which read `IRC_1002_C.bin` by name.

## Producers and consumers

- Producers: mrDIMAS's hrir_sphere_builder, from the IRCAM LISTEN compensated WAV files.
- Consumers: `tfar_hrtf` (its `HrirLibrary::load` at plugin start, and its tests through
  `CARGO_MANIFEST_DIR`). The plugins need the file beside them; `cargo xtask dist` does not copy
  it today.

## Boundaries

- Depends on: the sphere format of the `hrtf` crate pinned in the root `Cargo.toml` (0.9.0).
- Used by: `tfar_hrtf`.
- Rules: the sphere file is never edited; CREDITS.md stays beside it; `.gitattributes` keeps
  `*.bin` binary.

## Related documentation

- [Library survey](/documentation/program/research/library_survey.md) — the `hrtf` crate and the
  HRIR spheres it reads.
- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — where the HRTF sits in the voice
  chain.
