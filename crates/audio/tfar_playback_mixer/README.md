# Playback mixer

The `tfar_playback_mixer` crate: loading and mixing of the radio sound WAV files and the own-voice
stream into the outgoing audio. Used by `tfar_plugin_0_9`, `tfar_plugin_1_0`,
`tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0`.

## How it works

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

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_playback_mixer  # the unit tests
```

## Configuration

None: the crate reads no setting. The plugins pass the WAV paths under their plugin folder.

## Public surface

- `WaveLibrary`: `new`, `load`, `preload`, `cached`, `len`, `is_empty`.
- `Playback`: `new`, `from_stereo`, `id`, `frames`, `remaining_frames`, `is_finished`,
  `samples`; `PlaybackId`: `next`, `value`.
- `PlaybackMixer`: `new`, `add`, `release_finished`, `append_stream` (non-audio callers),
  `stream_producer`, `mix_into`, `active_playback_count`, `stream_queued_frames`; `MAX_ACTIVE_PLAYBACKS`,
  `PLAYBACK_QUEUE_CAPACITY`, `MAX_STREAMS`, `STREAM_CAPACITY_FRAMES`.
- `StreamProducer`: `name`, `push` (audio thread: never blocks or allocates).
- `Error`, `Result` and `prelude`.

## Boundaries

- Depends on: `tfar_sample_buffer` (the stereo buffer a playback's processing works on); `hound`
  for WAV decoding, `rtrb` for the queues to and from the audio thread, `parking_lot` for the short
  locks, `thiserror` for the error type.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_voice_pipeline_0_9`,
  `tfar_voice_pipeline_1_0`.
- Rules: a playback's filters and buffers are freed when it ends, off the audio thread;
  `mix_into` and `StreamProducer::push` never wait and never allocate
  (`a_producer_push_succeeds_while_another_thread_holds_every_mixer_lock`), and `mix_into` never
  writes past `channels * frames` samples or into channels 2 and up. Tier 1 in `crates/audio`: every workspace dependency has a lower tier and
  follows the category edge matrix (`cargo xtask verify crate-tiers`).

## Related documentation

- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — the sound playback section and
  the mixed-playback callback.
- [TFAR 0.9.12 plugin behaviour](/documentation/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
- [TFAR 1.0 plugin behaviour](/documentation/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
