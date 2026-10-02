**Status:** live

# README template: data

**When to use:** a folder of files that code reads rather than runs: the HRIR sphere in assets/hrir/, the
golden-vector fixtures the audio tests compare against, a scenario folder of the TeamSpeak mock. Test
fixtures inside a `tests/` folder are exempt and need no README. The
[README standard](/documentation/standards/readme_standard.md) defines every rule this template follows;
the data kind adds Format, and Producers and consumers.

## Skeleton

Copy the block and replace every `<…>` placeholder; each one says what goes there. A folder of one
homogeneous collection lists it as one glob line in Contents.

````markdown
# <What the data is, in plain words: no path, no backticks>

<One to three sentences: what the data is for and what reads it.>

## Contents

```text
<repository path of the folder>/
├── <child folder>/  <what it holds: a lowercase phrase, no closing period>
└── <file or glob>   <what it holds>
```

## How it works

<How the files are read and checked: the code or test that consumes them, when it reads them (compile
time, plugin start, a test run), and the invariants that hold across files. Leave the section out when
the folder has no child folders besides exempt ones and at most three files.>

## Format

- Encoding: <file type and encoding, and the naming convention>
- Layout: <the fields, sample format or structure the reading code expects, with the file that defines
  it>
- Adding a file: <where it goes, what it must hold, and the command that checks it>

## Producers and consumers

- Producers: <what writes the files (a person, a tool, an upstream source), with its path or URL>
- Consumers: <what reads them (crates, tests, dist), each with its path>

## Boundaries

- Depends on: <the types and sources the data must agree with>
- Used by: <everything outside the folder that reads it>
- Rules: <the invariants a change must keep: pinned sources, checksums, credits, and the test or gate
  that checks each>

## Related documentation

- [<document title>](/documentation/<path to the document>) — <what it covers>
````

## Worked sample

Written for assets/hrir/, as it reads once the sphere is added.

````markdown
# HRIR sphere

The head-related impulse responses the HRTF crate convolves each voice with, so a speaker is heard from
their direction. The set is the IRCAM sphere in the `hrtf` crate's binary format, with its credits.

## Contents

```text
assets/hrir/
├── CREDITS.md   where the measurements come from and the terms they are used under
└── *.bin        the HRIR sphere in the hrtf crate's format, one file per subject
```

## Format

- Encoding: the binary sphere format the `hrtf` crate reads, 48 kHz.
- Layout: points on a sphere around the listener, each with a left and a right impulse response.
- Adding a file: convert with the `hrtf` crate's own tooling, add its credit to `CREDITS.md`, and run
  `cargo test -p tfar_hrtf`.

## Producers and consumers

- Producers: the IRCAM measurements, converted once; nothing in the workspace writes these files.
- Consumers: `tfar_hrtf`, which loads the sphere when a plugin starts; `cargo xtask dist`, which copies
  it beside the plugin.

## Boundaries

- Depends on: the `hrtf` crate's sphere format.
- Used by: `tfar_hrtf`, both plugin apps through it, and dist.
- Rules: every file is credited in `CREDITS.md`; the sample rate matches TeamSpeak's 48 kHz.

## Related documentation

- [Audio decisions](/documentation/decisions/audio.md) — why the hrtf crate and this sphere.
````
