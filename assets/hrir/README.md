# HRIR sphere

The head-related impulse responses the binaural HRTF renders with: the IRCAM sphere in the `.bin`
format the `hrtf` crate reads. The folder holds no sphere yet; the sphere file and CREDITS.md, its
IRCAM credit, arrive together with the HRTF code, and nothing is downloaded before then.

## Contents

```text
assets/hrir/
```

## Format

- Encoding: one binary HRIR sphere file in the `hrtf` crate's `.bin` format, built at 44.1 kHz, and
  CREDITS.md beside it.
- Layout: the layout the `hrtf` crate's sphere loader defines; `tfar_hrtf` resamples it to 48 kHz
  at load.
- Adding a file: a new sphere comes with its credit in CREDITS.md and passes the `tfar_hrtf` tests.

## Producers and consumers

- Producers: mrDIMAS's hrir_sphere_builder, from the IRCAM Listen HRIR set.
- Consumers: `tfar_hrtf`, at plugin start; `cargo xtask dist`, which copies the sphere next to the
  plugins.

## Boundaries

- Depends on: the sphere format of the `hrtf` crate pinned in the root `Cargo.toml`.
- Used by: `tfar_hrtf`.
- Rules: the sphere file is never edited; CREDITS.md stays beside it.

## Related documentation

- [Library survey](/documentation/program/research/library_survey.md) — the `hrtf` crate and the
  HRIR spheres it reads.
