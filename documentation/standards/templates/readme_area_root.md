**Status:** live

# README template: area root

**When to use:** the top of a code tree or a category folder that groups crates without being one:
crates/, each crates/<category>/ folder, tools/ and assets/. The
[README standard](/documentation/standards/readme_standard.md) defines every rule this template follows;
the area root kind adds Getting started.

## Skeleton

Copy the block and replace every `<…>` placeholder; each one says what goes there. A code README carries
no status line, so the block opens with the title.

````markdown
# <Name of the area, in plain words: no path, no backticks>

<One to three sentences: what the area is for and which crates it holds.>

## Contents

```text
<repository path of the folder>/
├── <crate folder>/  <the crate in one phrase: a lowercase phrase, no closing period>
└── <crate folder>/  <the crate in one phrase; entries run in case-insensitive name order>
```

## How it works

<How the crates fit together: which tier each sits on, who calls whom, what they share, and which
categories they may depend on. An ASCII diagram in a text block helps here. Name each crate's part in
one clause; its own README holds the detail.>

## Getting started

<The few commands, run from the repository root, that build and test the area, each with what to
expect.>

## Boundaries

- Depends on: <the categories and external crates the crates use>
- Used by: <the categories and apps that use these crates>
- Rules: <the invariants particular to the area, each with the gate or test that holds it; no
  repository-wide law restated>

## Related documentation

- [<document title>](/documentation/<path to the document>) — <what it covers>
````

## Worked sample

Written for crates/audio/, the audio category, from the crate list and edges of the program records.

````markdown
# Audio crates

The signal-processing crates both plugins share: the sample buffer, the filters, the compressor, the
HRTF, the distance and cone gains, and the playback of radio sound files. Nothing here knows TeamSpeak,
the game link or either TFAR version.

## Contents

```text
crates/audio/
├── tfar_dsp_filters/     Butterworth and RBJ filters in f64 Direct Form II Transposed, many channels
├── tfar_dynamics/        the feed-forward compressor the radio effects run before their filters
├── tfar_hrtf/            binaural direction through the hrtf crate, with a block adapter per source
├── tfar_playback_mixer/  WAV playback of radio sounds, cached and mixed into voice frames
├── tfar_sample_buffer/   interleaved 16-bit frames: saturating gain, mixing, channel conversion
└── tfar_spatial_audio/   the distance curve and the emitter cone gain
```

## How it works

`tfar_sample_buffer` sits at tier 0; every other audio crate builds on it at tier 1, and `tfar_hrtf` and
`tfar_spatial_audio` also use `tfar_geometry` from foundation. The version crates compose them into each
mod's radio effects and voice pipeline.

```text
tier 1   tfar_dsp_filters  tfar_dynamics  tfar_hrtf  tfar_spatial_audio  tfar_playback_mixer
tier 0   tfar_sample_buffer                     (foundation: tfar_geometry)
```

## Getting started

```bash
cargo test -p tfar_sample_buffer -p tfar_dsp_filters   # unit tests and golden comparisons
cargo xtask verify crate-tiers                         # tiers and category edges hold
```

## Boundaries

- Depends on: foundation; `sci-rs`, `biquad`, `hrtf`, `hound` and `glam`.
- Used by: version_0_9, version_1_0 and the plugin apps.
- Rules:
  - audio crates depend only on foundation and audio (`cargo xtask verify crate-tiers`);
  - each external DSP crate stays in its one crate (`cargo xtask verify dependency-firewalls`).

## Related documentation

- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — the chain each version builds.
- [Audio decisions](/documentation/decisions/audio.md) — HRTF, the gain model, the filters.
````
