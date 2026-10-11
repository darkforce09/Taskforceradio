# HRTF

The `tfar_hrtf` crate: binaural direction rendering over the IRCAM HRIR sphere through the hrtf
crate. Used by `tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_voice_pipeline_0_9`,
`tfar_voice_pipeline_1_0`.

## How it works

The crate replaces clunk's ear delay with real binaural rendering through the `hrtf` crate.

- **Loading.** `HrirLibrary::load` reads `assets/hrir/IRC_1002_C.bin`, refuses a malformed file
  (the `hrtf` crate would abort the process on a zero sample rate, a bad index or a short file),
  lets the crate resample the 512-sample responses to 48 kHz (558 samples) and builds one
  prepared processor. Each `HrtfSource` copies it (about 2 MB), so sources share nothing mutable.
- **Block adapter.** The processor renders `INTERPOLATION_STEPS × CONVOLUTION_BLOCK_FRAMES`
  = 4 × 120 = 480 frames at a time and glides direction and gain across the four steps. The
  adapter is an input FIFO (the block being filled) and an output FIFO (the block rendered before
  it) drained at the same position: when the input block is full the output block is used up,
  the block is rendered, and both restart. Any frame sizes (480, 960, 441, 1) pass with no sample
  dropped or repeated and a constant added latency of exactly **480 frames, 10 ms**
  (`ADDED_LATENCY_FRAMES`); only the first 480 frames of a source (and after `reset`) are silence.
- **Idle restart.** A source fed nothing for more than `IDLE_RESET_AFTER` (200 ms) resets itself
  on its next frame, so the pending block and tails of one utterance never replay at the start of
  the next. The rule runs on the audio thread at the frame that ends the gap, so it never cuts
  into a running utterance the way a reset from another thread can.
- **Down-mix.** Stereo input becomes `(left + right) / 2`. The originals fed only the left
  channel to the direction stage; for the duplicated mono voice the mean is the same, and where
  the channels differ it keeps both.
- **Directions.** `ListenerRelativeDirection::from_arma` turns Arma's x east, y north, z up and
  the compass heading into (right, forward, up); the sphere is right-handed with x right, y up,
  z backwards, so the sampling vector is `(right, up, −forward)`. A reversal of more than about
  154° between two blocks snaps instead of gliding, so the crate's interpolated vector never
  shrinks toward the sphere's centre.
- **Level.** The compensated IRCAM set is diffuse-field equalised, so a frontal source comes out
  about 4 dB below its input and a source behind about 9 dB below; the distance curve and cone
  of `tfar_spatial_audio` set the level on top.

`hrir_library.rs` reads the file, passes it through `sphere_file_check.rs` (which refuses every
input that would make the `hrtf` crate panic), lets the crate resample and transform it, and keeps
one prepared `hrtf::HrtfProcessor`. `hrtf_source.rs` copies that processor per source and pairs it
with `block_adapter.rs`: each input sample goes into the pending block at the position the output
sample comes out of the rendered block, and a full pending block is rendered by the source's
renderer (direction and gain from the latest call, gliding from the previous block's). The numbers
the three agree on live in `render_timing.rs`. `listener_relative_direction.rs` holds the only
axis convention: Arma (x east, y north, z up, compass heading) to the listener's (right, forward,
up) to the sphere's (x right, y up, z backwards). Each module attaches its sibling test file in
`tests/` with `#[path]`; `lib.rs` attaches `tests/real_sphere.rs` under `cfg(test)`.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_hrtf  # the unit tests, over the real sphere in assets/hrir/
```

## Configuration

None: the crate reads no setting. The caller passes the sphere path to `HrirLibrary::load`.

## Public surface

- `HrirLibrary`: `load(path) -> Result<Arc<HrirLibrary>>`, `hrir_length`, `point_count`,
  `recorded_sample_rate_hz`, `source_path`.
- `HrtfSource`: `new(&HrirLibrary)`, `process(&StereoBuffer, &mut StereoBuffer, direction, gain)`,
  `process_mono(&[i16], &mut StereoBuffer, direction, gain)`, `reset()`.
- `ListenerRelativeDirection`: `from_arma`, `from_listener_axes`, `STRAIGHT_AHEAD`, `right`,
  `forward`, `up`.
- `SAMPLE_RATE_HZ`, `CONVOLUTION_BLOCK_FRAMES`, `INTERPOLATION_STEPS`, `RENDER_BLOCK_FRAMES`,
  `ADDED_LATENCY_FRAMES`, `IDLE_RESET_AFTER`; `Error`, `Result`, `prelude`.

## Boundaries

- Depends on: `tfar_geometry` (positions and angles); `tfar_sample_buffer` (the stereo buffer and
  the `i16`/`f32` conversion); `hrtf` (sphere parsing, resampling, convolution); `glam` (the
  listener-frame vector); `thiserror` for the error type.
- Used by: `tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_voice_pipeline_0_9`,
  `tfar_voice_pipeline_1_0`.
- Rules: a source's state belongs to one speaker and is never shared between two; `load` and
  `HrtfSource::new` allocate and run off the audio thread where possible, `process`,
  `process_mono` and `reset` allocate nothing. Tier 1 in `crates/audio`: every workspace
  dependency has a lower tier and follows the category edge matrix
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — where the HRTF sits in both
  voice chains.
- [HRIR sphere](/assets/README.md) — the sphere file, its format and its credits.
- [TFAR 1.0 plugin behaviour](/documentation/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
