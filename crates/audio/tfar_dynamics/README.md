# Dynamics

The `tfar_dynamics` crate: the feed-forward compressor the radio effects run once per frame. Used by
`tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_voice_pipeline_0_9`,
`tfar_voice_pipeline_1_0`.

## Contents

```text
crates/audio/tfar_dynamics/
├── Cargo.toml  the `tfar_dynamics` package (lib): tier 1, workspace lints and dependencies
└── src/        the compressor, its settings and presets, and the attack and release envelope
```

## How it works

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

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_dynamics  # the unit tests
```

## Configuration

None: the crate reads no setting. Each caller picks a `CompressorSettings` preset or fills one in.

## Public surface

- `Compressor`: `new`, `settings`, `reset`, `over_threshold_db`, `process_frame`,
  `process_stereo_i16`.
- `CompressorSettings` (`sample_rate_hz`, `threshold_db`, `ratio`, `attack_ms`, `release_ms`)
  with `VERSION_0_9`, `VERSION_1_0` and `validate`.
- `DC_OFFSET`, `Error`, `Result`, `CRATE_ROLE` and `prelude`.

## Boundaries

- Depends on: `tfar_sample_buffer` (the stereo `i16` frame type); `thiserror` for the error type.
- Used by: `tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_voice_pipeline_0_9`,
  `tfar_voice_pipeline_1_0`.
- Rules: one `process_stereo_i16` call is one pass over the block; the caller never runs a block
  through it twice. Processing allocates nothing. Tier 1 in `crates/audio`: every workspace
  dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — where the compressors sit in
  the 0.9 and 1.0 voice chains.
- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
