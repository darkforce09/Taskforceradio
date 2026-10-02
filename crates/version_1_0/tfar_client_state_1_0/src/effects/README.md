# Sender effects

The DSP objects each sender owns, keyed as the original keys them, and the requirements that
create them off the audio thread.

## Contents

```text
crates/version_1_0/tfar_client_state_1_0/src/effects/
├── client_effects.rs       `ClientEffects`: the per-key effects, filters, HRTF sources, compressors
├── dsp_requirement.rs      `DspRequirement` (one object to create) and `DspContext` (what with)
├── filter_designs.rs       the filter designs, the effect keys, `object_interception_cutoff_hz`
├── object_interception.rs  the object-interception low-pass slots, `ObjectInterceptionFilter`
└── tests/                  unit tests of `ClientEffects`
```

## How it works

`ClientEffects::ensure_dsp(requirements, context)` creates the two compressors (direct speech and
radio, so each envelope advances once per frame), reserves the object-interception buffer for
`frames_per_block`, and creates every required object that does not exist yet: radio effects by
radio id, HRTF sources by key (the HRIR sphere comes in `DspContext`), the can't-speak, vehicle
(per loss), speaker and phone filters by key, and the object-interception low-pass per object
count at the given strength. The audio path then borrows each object with a `*_mut(key)`
accessor, a `HashMap` lookup by `&str` that allocates nothing. `reset_radio_effects` and
`reset_voices` drop what the original's `resetRadioEffect` and `resetVoices` drop (the airborne
effects too).

The object-interception slots remember the strength they were designed with and are redesigned
when it changes; a cutoff `2000 − min(count, 5) · strength` at or below 0 Hz is
`ObjectInterceptionFilter::Silence`.

## Boundaries

- Depends on: `tfar_radio_effects_1_0`, `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`,
  `tfar_sample_buffer`; the crate's `error`.
- Used by: `ClientData::effects`; the voice pipeline through the crate root.
- Rules: only `ensure_dsp` and the resets allocate
  (`ensure_creates_each_object_once_and_keeps_it`); two compressors per sender
  (`direct_speech_and_radio_each_own_a_compressor`); a non-positive cutoff is silence
  (`an_object_cutoff_at_or_below_zero_is_silence`).
