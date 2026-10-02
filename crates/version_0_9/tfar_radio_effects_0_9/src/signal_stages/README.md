# Signal stages 0.9

The per-sample building blocks the 0.9.12 effects chain together.

## Contents

```text
crates/version_0_9/tfar_radio_effects_0_9/src/signal_stages/
├── delay_line.rs       `DelayLine`: 2400 slots, a sample comes back 2399 samples later
├── float_narrowing.rs  `narrow_to_f32`: a filtered `f64` sample back to `f32`
├── foldback.rs         `foldback` and the level-driven `foldback_threshold`
├── ring_modulator.rs   `RingModulator`: carrier `sin(phase · π/2)`, phase step 90/48000, reset above 1
├── sample_dropout.rs   `SampleDropout`: seeded draws in 0..=32767 below `trunc(32767 · error)`
└── tests/              the delay, the fold bounds, the carrier ramp, the drop share and the seed
```

## How it works

Each stage is a small value with the state the original keeps in its effect object: the delay
line's slots and position, the ring modulator's phase, the dropout's generator. The stages hold no
filters; the effects in `effects/` add those. The dropouts replace the original's global, unseeded
`rand()` with one `fastrand` generator per effect, seeded by its creator, so runs are reproducible
and two effects never share a sequence; the draw keeps the 0..=32767 range of the Windows `rand()`.

## Boundaries

- Depends on: `fastrand` (`Rng`).
- Used by: `effects/`.
- Rules: no stage allocates after construction or panics; the delay is exactly 2399 samples and the
  fold stays inside ±threshold (the tests in `tests/`).
