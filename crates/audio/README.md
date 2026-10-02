# Audio crates

The audio building blocks both plugin versions share: the sample buffer, the filters, the
compressor, the HRTF, the distance and cone gains and the radio sound mixer.

## Contents

```text
crates/audio/
├── tfar_dsp_filters/     tier 1: the Butterworth and RBJ filters
├── tfar_dynamics/        tier 1: the compressor
├── tfar_hrtf/            tier 1: binaural direction rendering
├── tfar_playback_mixer/  tier 1: radio sound playback
├── tfar_sample_buffer/   tier 0: the shared sample frame type and saturating gain
└── tfar_spatial_audio/   tier 1: the distance curve and cone gains
```

## How it works

Every crate works on the `tfar_sample_buffer` frame type. `tfar_dsp_filters` and `tfar_dynamics`
shape a radio signal, `tfar_hrtf` and `tfar_spatial_audio` place a voice in space, and
`tfar_playback_mixer` adds the radio sounds. The version tracks compose them into a voice pipeline;
no crate here knows which mod version calls it.

## Getting started

Run from the repository root:

```bash
# the unit tests of every crate in this category
cargo test \
  -p tfar_dsp_filters \
  -p tfar_dynamics \
  -p tfar_hrtf \
  -p tfar_playback_mixer \
  -p tfar_sample_buffer \
  -p tfar_spatial_audio
```

## Boundaries

- Depends on: `crates/foundation` and the audio category itself.
- Used by: the radio effect, client state and voice pipeline crates of both version tracks, and the
  plugin apps.
- Rules: an audio crate depends only on foundation and audio crates; `sci-rs` and `biquad` stay in
  `tfar_dsp_filters`, `hrtf` in `tfar_hrtf` and `hound` in `tfar_playback_mixer`
  (`cargo xtask verify crate-tiers` holds the edges, `cargo xtask verify dependency-firewalls` the
  external crates); each crate declares `category = "crates/audio"` and its tier: 0 without a
  workspace dependency, else one above the highest.

## Related documentation

- [Library survey](/documentation/program/research/library_survey.md) — the external crates chosen for each need.
