# Audio

The voice DSP both plugins share, one module per stage: `sample_buffer` (TeamSpeak's interleaved
16-bit frames and saturating gain), `dsp_filters` (Butterworth and RBJ filters), `dynamics` (the
compressor), `hrtf` (binaural rendering over the IRCAM sphere), `spatial_audio` (distance and cone
gains, stereo pan) and `playback_mixer` (radio sounds and the own-voice stream). The version tracks
compose them into a voice pipeline; no module knows which mod version calls it. Logging goes only
through the `tracing` macros; the plugins install the subscriber.

```bash
cargo test -p tfar_audio
cargo test -p tfar_golden_vectors   # the modules against the original C/C++ DSP
```

Boundaries: depends on `tfar_foundation`; used by the version libraries, both plugins and the
golden-vector tool. `sci-rs`, `biquad`, `hrtf` and `hound` stay in this crate
(`cargo xtask verify dependency-firewalls`).

## `sample_buffer`

The `tfar_audio::sample_buffer` crate: interleaved 16-bit sample frames as TeamSpeak hands them out, with
saturating gain. Used by `tfar_audio::dsp_filters`, `tfar_audio::dynamics`, `tfar_audio::hrtf`, `tfar_audio::playback_mixer`,
`tfar_plugin_logic_0_9::radio_effects`, `tfar_plugin_logic_1_0::radio_effects`, `tfar_audio::spatial_audio`, `tfar_plugin_logic_0_9::voice_pipeline`,
`tfar_plugin_logic_1_0::voice_pipeline`.

### How it works

A TeamSpeak voice callback hands over an interleaved `i16` block of 1, 2 or more channels.
`copy_to_stereo` checks the block against its channel and frame counts and copies it into a
`StereoBuffer` (mono duplicated, channels 0 and 1 otherwise); the pipeline works on that buffer,
going through `to_float_planar` and `from_float_planar` around the `f32` filters and effects, with
`apply_gain`, `apply_stereo_gain` and `mix_into_additive` saturating at the `i16` limits; and
`write_back_from_stereo` folds the result back (mono as the average of left and right, channels 2
and up silenced). The scales follow the originals: `i16` to `f32` divides by 32767, `f32` to `i16`
clamps to [-1, 1] and multiplies by 32766. Once a `StereoBuffer` is reserved for the largest
block, nothing on these paths allocates.

`lib.rs` declares the modules privately and re-exports their public items at the crate root;
`prelude.rs` re-exports the same items except `Error` and `Result`, so a glob import never shadows
`std`'s `Result`. `StereoBuffer` exposes its frames to the other modules as `[i16; 2]` pairs, so
gain, the adapters and the conversions walk frames without indexing. Every `f32` to `i16` step
goes through `sample_saturation.rs`, whose `as` cast truncates toward zero and saturates. Each
module attaches its sibling test file in `tests/` with `#[path]`.

### Configuration

None: the crate reads no setting.

### Public surface

- `StereoBuffer`: `with_frames`, `frames`, `capacity_frames`, `reserve_frames`, `set_frames`,
  `clear`, `as_slice`, `as_mut_slice`, `copy_from`, `CHANNELS`.
- `copy_to_stereo`, `write_back_from_stereo`, `mono_to_stereo`: the TeamSpeak channel adapters.
- `apply_gain`, `apply_stereo_gain`, `mix_into_additive`: saturating gain and mixing.
- `INPUT_SCALE`, `OUTPUT_SCALE`, `sample_to_float`, `float_to_sample`, `to_float_planar`,
  `from_float_planar`: float conversion.
- `Error`, `Result` and `prelude`.

### Boundaries

- Depends on: no workspace crate; `thiserror` for the error type.
- Used by: `tfar_audio::dsp_filters`, `tfar_audio::dynamics`, `tfar_audio::hrtf`, `tfar_audio::playback_mixer`,
  `tfar_plugin_logic_0_9::radio_effects`, `tfar_plugin_logic_1_0::radio_effects`, `tfar_audio::spatial_audio`, `tfar_plugin_logic_0_9::voice_pipeline`,
  `tfar_plugin_logic_1_0::voice_pipeline`.
- Rules: gain and mixing saturate at the `i16` limits and never wrap; no adapter reads or writes
  past `channels * frames` samples; only `StereoBuffer::with_frames` and `reserve_frames` allocate
  by design.

## `dsp_filters`

The `tfar_audio::dsp_filters` crate: Butterworth and RBJ filters in f64 Direct Form II Transposed, with
per-channel state. Used by `tfar_plugin_logic_0_9::client_state`, `tfar_plugin_logic_1_0::client_state`,
`tfar_plugin_logic_0_9::radio_effects`, `tfar_plugin_logic_1_0::radio_effects`, `tfar_plugin_logic_0_9::voice_pipeline`,
`tfar_plugin_logic_1_0::voice_pipeline`.

### How it works

A caller describes a filter as a `FilterDesign` and builds a `ChannelFilter<CHANNELS>` from it
once, off the audio thread: the Butterworth designs come from `sci-rs` (`butter_dyn`, the port of
`scipy.signal.butter`) as second-order sections, the RBJ designs from `biquad`'s cookbook
coefficients, and both become a `SecondOrderSections` cascade of normalised sections. The filter
then runs every channel through its own copy of the cascade, each section a `biquad`
`DirectForm2Transposed<f64>`, on the TeamSpeak audio thread without allocating:
`process_frame`, `process_frames_f64` and `process_frames_f32` work on interleaved frames,
`MonoFilter::process_sample` on the radio effects' mono signal, and
`StereoFilter::process_stereo_i16` is the original `processFilterStereo`
(`reference/tfar_1_0/ts/src/helpers.hpp`): `sample / 32767`, filter, `× gain`; above 1.0 the sample
is `i16::MAX`, below -1.0 `i16::MIN`, otherwise `× 32766` and truncated, as the original clips.

#### Designs and their originals

| `FilterDesign` | DSPFilters original | TFAR use |
|---|---|---|
| `ButterworthLowPass { order: 2, .. }` | `Butterworth::LowPass<2>` | vehicle muffling, object interception |
| `ButterworthLowPass { order: 4, .. }` | `Butterworth::LowPass<4>` | can't speak and underwater, 100 Hz |
| `ButterworthBandPass { order: 1, .. }` | `Butterworth::BandPass<1>` | speaker, 2000 / 1000 |
| `ButterworthBandPass { order: 2, .. }` | `Butterworth::BandPass<2>` | phone 1850 / 1550, underwater 1000 / 400 |
| `RbjHighPass`, `RbjLowPass` | `RBJ::HighPass`, `RBJ::LowPass` | the radio effects |

#### Band-pass centre and width

DSPFilters' `BandPassBase::setup(order, sampleRate, centre, width)` places the digital band edges
at `centre ∓ width / 2` (`BandPassTransform` in
`reference/tfar_1_0/ts/include/dspfilters/source/PoleFilter.cpp`) and normalises the peak to 1 at
the warped geometric centre `2·atan(sqrt(tan(ω1/2)·tan(ω2/2)))`. This crate passes the same two
edges to `butter_dyn` as the band-pass critical frequencies, which puts the −3 dB points exactly
on them and the unit peak at the same warped centre. For TFAR's band-passes the edges are
1500–2500 Hz (speaker), 1075–2625 Hz (phone) and 800–1200 Hz (underwater); the tests check the
gain at both edges is −3.0103 dB: to 1e-9 in the designed magnitude, and to 0.01 dB when a sine is
measured through the filter.

`FilterDesign::second_order_sections` dispatches each variant to `designs/`, which checks the
parameters and calls `sci-rs` or `biquad`; the result is a `SecondOrderSections` of normalised,
finite sections. `ChannelFilter::from_sections` turns each section into a `biquad`
`DirectForm2Transposed<f64>` and keeps one chain of them per channel; every processing method folds
a sample through its channel's chain. `lib.rs` re-exports the public items at the crate root and
`prelude.rs` the same items except `Error` and `Result`. Each module attaches its sibling test file
in `tests/` (or `designs/tests/`) with `#[path]`.

### Modules

#### `src/designs/`

Each design checks its parameters with `parameter_checks.rs` first, using the normalised frequency
`2 · f / fs` that `sci-rs` computes, so the `panic!`s inside `butter_dyn` (a frequency outside
(0, Nyquist), band edges out of order) cannot be reached. `butterworth_design.rs` derives the
band-pass edges as `centre ∓ width / 2`, DSPFilters' convention, and asks `butter_dyn` for
second-order sections; `rbj_design.rs` asks `biquad::Coefficients::from_params` for the cookbook
section DSPFilters' `RBJ` filters compute. Both normalise through
`SecondOrderSection::from_unnormalised`, which refuses a zero leading coefficient or a coefficient
that is not finite.

### Configuration

None: the crate reads no setting.

### Public surface

- `FilterDesign` (`ButterworthLowPass`, `ButterworthBandPass`, `RbjLowPass`, `RbjHighPass`) with
  `second_order_sections` and `band_edges_hz`; `MAX_BUTTERWORTH_ORDER`.
- `SecondOrderSections` (`new`, `sections`, `sample_rate_hz`, `magnitude_at`) and
  `SecondOrderSection` (`b0`, `b1`, `b2`, `a1`, `a2`, `from_unnormalised`, `coefficients`).
- `ChannelFilter<CHANNELS>` (`new`, `from_sections`, `sections`, `reset`, `process_frame`,
  `process_frames_f64`, `process_frames_f32`); `MonoFilter` (`process_sample`) and
  `StereoFilter` (`process_stereo_i16`).
- `Error`, `Result` and `prelude`.

### Boundaries

- Depends on: `tfar_audio::sample_buffer` (the stereo buffer and the `i16`/`f32` scales); `sci-rs`
  (Butterworth design), `biquad` (RBJ coefficients and the Direct Form II Transposed section) and
  `thiserror`.
- Used by: `tfar_plugin_logic_0_9::client_state`, `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_logic_0_9::radio_effects`,
  `tfar_plugin_logic_1_0::radio_effects`, `tfar_plugin_logic_0_9::voice_pipeline`, `tfar_plugin_logic_1_0::voice_pipeline`.
- Rules: coefficients are designed once at construction, off the audio thread; processing
  allocates nothing and never panics; every parameter is checked before a design library is
  called, so their panics are unreachable.

## `dynamics`

The `tfar_audio::dynamics` crate: the feed-forward compressor the radio effects run once per frame. Used by
`tfar_plugin_logic_0_9::client_state`, `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_logic_0_9::voice_pipeline`,
`tfar_plugin_logic_1_0::voice_pipeline`.

### How it works

`Compressor` is the stereo-linked SimpleComp of ChunkWare SimpleSource 1.12
(`reference/tfar_1_0/ts/include/simpleSource/`), step for step: per frame the key is the louder
of `|left|` and `|right|` plus `DC_OFFSET` (1e-25), converted to decibels with
`ln · 20 / ln 10`; the excess `max(0, key_dB − threshold_dB)` plus `DC_OFFSET` runs through a
one-pole envelope with the attack coefficient `exp(−1000 / (attack_ms · rate))` when it rises and
the release coefficient when it falls; the offset comes off again, and both channels are
multiplied by `dB2lin(excess · (ratio − 1))`. `process_stereo_i16` feeds the raw `i16` values as
`f64`, as the originals' `processCompressor` does, and truncates the result back toward zero,
saturating at the `i16` limits.

The threshold is therefore on the raw sample scale: full scale (32767) is 90.3 dB. The presets
are the originals' settings: `CompressorSettings::VERSION_0_9` (65 dB, ratio 0.1, attack 1 ms,
release 300 ms, 48 kHz) and `CompressorSettings::VERSION_1_0` (the same at 80 dB). Measured on a
full-scale step at 48 kHz, the envelope reaches 1 − 1/e of its excess after 48 frames (1 ms) and
falls to 1/e after 14 400 frames (300 ms); in steady state the output sits at
`threshold + ratio · excess`, 81.03 dB (11 260) for a full-scale input under the 1.0 preset.
Below the threshold, with the envelope at rest, the gain is exactly 1.

Credit: the compressor follows SimpleSource 1.12, © 2006 ChunkWare Music Software, released open
source under an MIT-style permission notice, which the original files under
`reference/tfar_1_0/ts/include/simpleSource/` carry in full.

`Compressor::new` checks the `CompressorSettings` and builds an `AttackReleaseEnvelope` from the
attack and release time constants; `process_frame` turns the louder channel into a decibel excess
over the threshold, runs it through the envelope against the compressor's own envelope state, and
scales both channels by the resulting gain. `lib.rs` re-exports the public items at the crate root
and `prelude.rs` the same items except `Error` and `Result`. Each module attaches its sibling test
file in `tests/` with `#[path]`.

### Configuration

None: the crate reads no setting. Each caller picks a `CompressorSettings` preset or fills one in.

### Public surface

- `Compressor`: `new`, `settings`, `reset`, `over_threshold_db`, `process_frame`,
  `process_stereo_i16`.
- `CompressorSettings` (`sample_rate_hz`, `threshold_db`, `ratio`, `attack_ms`, `release_ms`)
  with `VERSION_0_9`, `VERSION_1_0` and `validate`.
- `DC_OFFSET`, `Error`, `Result` and `prelude`.

### Boundaries

- Depends on: `tfar_audio::sample_buffer` (the stereo `i16` frame type); `thiserror` for the error type.
- Used by: `tfar_plugin_logic_0_9::client_state`, `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_logic_0_9::voice_pipeline`,
  `tfar_plugin_logic_1_0::voice_pipeline`.
- Rules: one `process_stereo_i16` call is one pass over the block; the caller never runs a block
  through it twice. Processing allocates nothing.

## `hrtf`

The `tfar_audio::hrtf` crate: binaural direction rendering over the IRCAM HRIR sphere through the hrtf
crate. Used by `tfar_plugin_logic_0_9::client_state`, `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_logic_0_9::voice_pipeline`,
`tfar_plugin_logic_1_0::voice_pipeline`.

### How it works

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
  of `tfar_audio::spatial_audio` set the level on top.

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

### Configuration

None: the crate reads no setting. The caller passes the sphere path to `HrirLibrary::load`.

### Public surface

- `HrirLibrary`: `load(path) -> Result<Arc<HrirLibrary>>`, `hrir_length`, `point_count`,
  `recorded_sample_rate_hz`, `source_path`.
- `HrtfSource`: `new(&HrirLibrary)`, `process(&StereoBuffer, &mut StereoBuffer, direction, gain)`,
  `process_mono(&[i16], &mut StereoBuffer, direction, gain)`, `reset()`.
- `ListenerRelativeDirection`: `from_arma`, `from_listener_axes`, `STRAIGHT_AHEAD`, `right`,
  `forward`, `up`.
- `SAMPLE_RATE_HZ`, `CONVOLUTION_BLOCK_FRAMES`, `INTERPOLATION_STEPS`, `RENDER_BLOCK_FRAMES`,
  `ADDED_LATENCY_FRAMES`, `IDLE_RESET_AFTER`; `Error`, `Result`, `prelude`.

### Boundaries

- Depends on: `tfar_foundation::geometry` (positions and angles); `tfar_audio::sample_buffer` (the stereo buffer and
  the `i16`/`f32` conversion); `hrtf` (sphere parsing, resampling, convolution); `glam` (the
  listener-frame vector); `thiserror` for the error type.
- Used by: `tfar_plugin_logic_0_9::client_state`, `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_logic_0_9::voice_pipeline`,
  `tfar_plugin_logic_1_0::voice_pipeline`.
- Rules: a source's state belongs to one speaker and is never shared between two; `load` and
  `HrtfSource::new` allocate and run off the audio thread where possible, `process`,
  `process_mono` and `reset` allocate nothing.

## `spatial_audio`

The `tfar_audio::spatial_audio` crate: the distance curve and emitter cone gains that replace X3DAudio,
plus the simple stereo panner both versions fall back to. Used by `tfar_plugin_logic_0_9::voice_pipeline`,
`tfar_plugin_logic_1_0::voice_pipeline`.

### How it works

The 1.0 plugin got level and direction for direct speech from X3DAudio. Here `tfar_audio::hrtf` gives
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
`tfar_foundation::geometry::Direction`. Each module attaches its sibling test file in `tests/` with `#[path]`.

### Configuration

None: the crate reads no setting. The 1.0 `voiceCone` setting decides in the voice pipeline
whether the cone applies.

### Public surface

- `DistanceCurve`: `new`, `tfar_voice`, `points`, `gain_at`, `MAX_POINTS`; `CurvePoint`.
- `EmitterCone`: `new`, `TFAR_VOICE`, `gain`, `inner_angle`, `outer_angle`, `inner_volume`,
  `outer_volume`.
- `stereo_pan_gains`, `apply_pan`, `ILD_COEFFICIENT_0_9`, `ILD_COEFFICIENT_1_0`, `ILD_BASE_GAIN`:
  the simple interaural level difference.
- `Error`, `Result` and `prelude`.

### Boundaries

- Depends on: `tfar_foundation::geometry` (directions and angles); `tfar_audio::sample_buffer` (the stereo buffer
  and saturating stereo gain); `thiserror` for the error type.
- Used by: `tfar_plugin_logic_0_9::voice_pipeline`, `tfar_plugin_logic_1_0::voice_pipeline`.
- Rules: every gain is finite; nothing allocates, construction included, so curves and cones may
  be built on the TeamSpeak audio thread.

## `playback_mixer`

The `tfar_audio::playback_mixer` crate: loading and mixing of the radio sound WAV files and the own-voice
stream into the outgoing audio. Used by `tfar_plugin_0_9`, `tfar_plugin_1_0`,
`tfar_plugin_logic_0_9::voice_pipeline`, `tfar_plugin_logic_1_0::voice_pipeline`.

### How it works

`WaveLibrary` decodes a radio sound with `hound` once and caches it by canonical path as a shared
`Arc<[i16]>`; anything but 48 kHz, 2-channel, 16-bit integer PCM is an `Error` naming the file and
its format. A `Playback` copies a sound and runs the caller's processing (gain, filters, panning)
on it at creation, off the audio thread, so the closure and the filters it owns are freed before
the playback is queued; it is named by a `PlaybackId` from a counter, never `rand()`.
`PlaybackMixer::add` queues it through an `rtrb` ring buffer. `PlaybackMixer::mix_into` is the
mixed-playback callback: it zeroes channels 0 and 1 where the fill mask says TeamSpeak left them
empty, adds every active playback and stream into them saturating (a mono block gets the average
of left and right), leaves channels 2 and up alone, sets the written channels' mask bits, and
hands finished playbacks back through a second ring buffer so `add` or `release_finished` frees
them off the audio thread. It takes its locks with `try_lock` and never allocates. The
`my_radio_voice` stream of the player's own radio voice is a named FIFO filled by `append_stream`
and bounded to 500 ms, dropping the oldest frames so a stalled playback side never grows memory
or delay. `append_stream` takes a blocking lock, so it is for callers off the audio threads; the
captured-voice callback feeds a stream through a `StreamProducer` from
`PlaybackMixer::stream_producer` instead: an `rtrb` ring of the same 500 ms whose `push` never
blocks or allocates, even while another thread holds the mixer, drops the oldest frames when the
ring is full, and is drained in order by `mix_into`.

`lib.rs` declares the modules privately and re-exports their public items at the crate root;
`prelude.rs` re-exports the same items except `Error` and `Result`, so a glob import never
shadows `std`'s `Result`. `output_block.rs` is the one place samples reach the TeamSpeak block:
playbacks and streams both feed it an iterator of interleaved stereo samples, so a stream's ring
of samples never has to be made contiguous. `PlaybackMixer` keeps four locks: the hand-off of new
and finished playbacks (non-audio threads), the mixing state (the audio thread, `try_lock`), the
streams (`append_stream` locks, `mix_into` uses `try_lock`) and the list of producer streams
(`stream_producer` and `release_finished` lock, `mix_into` uses `try_lock`). A `StreamProducer`
writes its own ring without any mixer lock; it shares the ring's reading side with the mixer
behind a lock both take only with `try_lock`, the mixer to drain and the producer to drop the
oldest frames when the ring is full. Each module attaches its sibling
test file in `tests/` with `#[path]`.

### Configuration

None: the crate reads no setting. The plugins pass the WAV paths under their plugin folder.

### Public surface

- `WaveLibrary`: `new`, `load`, `preload`, `cached`, `len`, `is_empty`.
- `Playback`: `new`, `from_stereo`, `id`, `frames`, `remaining_frames`, `is_finished`,
  `samples`; `PlaybackId`: `next`, `value`.
- `PlaybackMixer`: `new`, `add`, `release_finished`, `append_stream` (non-audio callers),
  `stream_producer`, `mix_into`, `active_playback_count`, `stream_queued_frames`; `MAX_ACTIVE_PLAYBACKS`,
  `PLAYBACK_QUEUE_CAPACITY`, `MAX_STREAMS`, `STREAM_CAPACITY_FRAMES`.
- `StreamProducer`: `name`, `push` (audio thread: never blocks or allocates).
- `Error`, `Result` and `prelude`.

### Boundaries

- Depends on: `tfar_audio::sample_buffer` (the stereo buffer a playback's processing works on); `hound`
  for WAV decoding, `rtrb` for the queues to and from the audio thread, `parking_lot` for the short
  locks, `thiserror` for the error type.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_plugin_logic_0_9::voice_pipeline`,
  `tfar_plugin_logic_1_0::voice_pipeline`.
- Rules: a playback's filters and buffers are freed when it ends, off the audio thread;
  `mix_into` and `StreamProducer::push` never wait and never allocate
  (`a_producer_push_succeeds_while_another_thread_holds_every_mixer_lock`), and `mix_into` never
  writes past `channels * frames` samples or into channels 2 and up.
