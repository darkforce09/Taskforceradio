**Status:** live

# README template: crate root

**When to use:** the folder that holds a library's or a tool's `Cargo.toml`: each crate under
crates/<category>/, and each tool under tools/ (xtask, the TeamSpeak mock, the extension probe, the
golden vectors). The four deliverables, in the version categories, use the
[app template](/documentation/standards/templates/readme_app.md) instead. The
[README standard](/documentation/standards/readme_standard.md) defines every rule this template follows;
the crate root kind adds Getting started, Configuration and Public surface.

## Skeleton

Copy the block and replace every `<…>` placeholder; each one says what goes there. A code README carries
no status line. A kind section with nothing to hold keeps its heading and says so in one line.

````markdown
# <Crate name, in plain words: no path, no backticks>

<One to three sentences: what the crate is, with its package name, and who uses it.>

## Contents

```text
<repository path of the folder>/
├── Cargo.toml  <the package: its category and tier, and what each dependency is for>
└── src/        <what the source tree holds, in one phrase>
```

## How it works

<What happens from the entry point to the result: the main types, the modules, the threads that call
them, and the invariants that span them. Name each child's part in one clause.>

## Getting started

<The commands, run from the repository root, that build and test the crate, each with what to expect.
Name the target triple when it is not the host's.>

## Configuration

<Every setting the crate reads: Cargo features, files, TeamSpeak or game settings, each with its default
and the file that reads it; or one line saying it reads none.>

## Public surface

- <module, type, function or binary>: <what it offers and who outside the crate uses it>

## Boundaries

- Depends on: <the workspace crates and external crates it uses, from its Cargo.toml>
- Used by: <every crate or tool that uses it, found with git grep>
- Rules: <the invariants a change here must keep, and the test or gate that checks each>

## Related documentation

- [<document title>](/documentation/<path to the document>) — <what it covers>
````

## Worked sample

Written for `tfar_sample_buffer`, the tier 0 audio crate, as its README reads once its code is written.

````markdown
# Sample buffer

The `tfar_sample_buffer` crate: the interleaved 16-bit sample frames TeamSpeak hands the voice
callbacks, with the gain, mixing and channel conversions every audio crate builds on.

## Contents

```text
crates/audio/tfar_sample_buffer/
├── Cargo.toml  the library package, tier 0 in crates/audio; its one dependency is thiserror
└── src/        the frame view, the gain and mix operations, the channel conversions
```

## How it works

A `SampleFrames` borrows one callback's samples together with its channel count, so a frame count and a
channel count always travel with the slice. `apply_gain` multiplies every sample and saturates at the
16-bit limits; a gain of 0 zeroes the frames and a gain of 1 leaves them untouched. `mix_into` adds one
buffer into another, saturating. `to_stereo` and `from_stereo` copy between a TeamSpeak channel layout
and the two-channel working buffer the effects use. Nothing allocates after a buffer's first use.

## Getting started

```bash
cargo test -p tfar_sample_buffer    # the unit tests, under a second
```

## Configuration

None: the crate reads no setting.

## Public surface

- `SampleFrames`, `StereoBuffer`: the borrowed and the owned frame types, used by every audio crate.
- `apply_gain`, `mix_into`, `to_stereo`, `from_stereo`: used by the radio effects and voice pipelines.

## Boundaries

- Depends on: `thiserror`; no workspace crate.
- Used by: `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`, `tfar_spatial_audio`,
  `tfar_playback_mixer`, both radio effects crates and both voice pipelines.
- Rules:
  - gain saturates and never wraps (`a_large_gain_saturates_at_the_sixteen_bit_limits` in
    `src/tests/gain.rs`);
  - a channel count of zero is an error value, never a division
    (`a_zero_channel_count_is_an_error`).

## Related documentation

- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — where the buffer sits in each
  version's chain.
````
