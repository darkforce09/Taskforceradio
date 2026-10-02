# Assets

Data files the deliverables load at run time and that are not code. The spell check skips them, and
git treats binary assets as binary.

## Contents

```text
assets/
└── hrir/  the IRCAM HRIR sphere for binaural HRTF, with its credits
```

## How it works

`tfar_hrtf` loads the HRIR sphere from `hrir/` and resamples it for TeamSpeak's 48 kHz audio, and
`cargo xtask dist` copies the assets the plugins need next to the built files. The folder holds no
asset file yet: none is downloaded until the HRTF code that reads it is written.

## Getting started

None: there is nothing to build or run here.

## Boundaries

- Depends on: nothing.
- Used by: `tfar_hrtf`.
- Rules: assets are data, never edited by hand; every third-party asset carries its credit beside it.

## Related documentation

- [Library survey](/documentation/program/research/library_survey.md) — the `hrtf` crate and the
  HRIR spheres it reads.
