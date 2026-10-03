# Playback mixer source

The source of `tfar_playback_mixer`: the WAV library, the playback and its id, the named stream,
the mixed-playback block helpers and the mixer, with the role constant, the error type and the
prelude.

## Contents

```text
crates/audio/tfar_playback_mixer/src/
├── crate_role.rs       `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs            `Error`, the `thiserror` enum, and `Result`
├── lib.rs              the crate root: module header, `mod` lines and re-exports
├── output_block.rs     the block bounds check, the fill-mask silencing and saturating stereo mixing
├── playback.rs         `Playback` and `PlaybackId`
├── playback_mixer.rs   `PlaybackMixer` and its capacity constants
├── playback_stream.rs  the bounded named FIFO behind `append_stream`, `STREAM_CAPACITY_FRAMES`
├── prelude.rs          the public API except `Error` and `Result`, for glob import
├── stream_producer.rs  `StreamProducer`: the wait-free `rtrb` ring an audio callback feeds a stream with
├── wave_library.rs     `WaveLibrary`, the `hound` decoder and cache
└── tests/              one unit test file per module
```

## How it works

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

## Public surface

- `WaveLibrary`, `Playback`, `PlaybackId`, `PlaybackMixer`, `StreamProducer`,
  `MAX_ACTIVE_PLAYBACKS`, `PLAYBACK_QUEUE_CAPACITY`, `MAX_STREAMS`, `STREAM_CAPACITY_FRAMES`,
  `CRATE_ROLE`, `Error`, `Result` and `prelude`, re-exported at the crate root.

## Boundaries

- Depends on: `tfar_sample_buffer`, `hound`, `rtrb`, `parking_lot`, `thiserror`.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_voice_pipeline_0_9`,
  `tfar_voice_pipeline_1_0`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` (`cargo xtask verify test-placement`).
