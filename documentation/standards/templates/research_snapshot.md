**Status:** live

# Template: research snapshot

**When to use:** a dated survey of the options for the needs of the port (crates, their versions and
licences, host behaviour, measurements), written once and then kept as it stands. Snapshots live in
`documentation/program/research/`, one file each, named in snake_case after the subject; a snapshot that
follows an earlier one on the same subject adds its date to the name (library_survey_2026_11_04.md).
Each is listed in the research folder's README. The
[documentation standards](/documentation/standards/documentation_standards.md) define frozen records and
their lifecycle.

## Skeleton

Copy the block and replace every `<…>` placeholder; each one says what goes there. The status line is
`**Status:** frozen record (YYYY-MM-DD)` with the day the facts were gathered: a snapshot is a frozen
record from its first commit. A need with too few options for a table gives them in one paragraph.

````markdown
**Status:** frozen record (<YYYY-MM-DD>)

# <Subject of the snapshot, in plain words>

<What the snapshot surveys, the date, where the facts come from (crates.io, repositories, SDK headers,
measurements), and what each marker in the tables means. Link the decision entries the survey serves.>

## 1. <Need>

| Option | <Version, date> | <the property that decides> | <Notes> |
|---|---|---|---|
| <crate or approach> | <version, release date> | <value> | <what matters for the port: limits, fit, licence> |

<Options ruled out and why, and anything a table cell cannot hold.>

**Recommendation:** <the option to use, the settings it needs, the fallback, and what must be measured
before relying on it>

## Recommended stack

| Need | Choice |
|---|---|
| <Need> | <the recommended option, with its alternative in brackets> |

## Hard gaps

- **<gap>:** <what no option covers, and what the port writes itself>

## Sources

[<source title>](<URL>) · [<source title>](<URL>)
````

A snapshot keeps its words. Only a broken link is fixed; new facts go into a new snapshot, and a decision
that changes because of them gets a new entry in the [decision log](/documentation/decisions/README.md).
A frozen record carries dates freely and may pass 500 lines.

## Worked sample

Abridged from the library survey behind decision D18: one need, the matching row of the stack, one gap.
The sample sits in a fenced block, so no gate reads it.

````markdown
**Status:** frozen record (2026-10-02)

# Rust libraries for the TFAR port

The crates.io crates that cover the port's needs, checked on 2026-10-02 against their latest releases.

## 1. Arma extension exports

| Option | Version | Takes the plain-string call | Notes |
|---|---|---|---|
| `arma-rs` | 1.x | no | looks the whole input up as a command name, so TFAR's tab-separated calls fail |
| own exports on `windows-sys` | 0.61.2 | yes | about 150 lines: `RVExtension`, `RVExtensionVersion`, truncating writes |

**Recommendation:** write the exports by hand in `tfar_arma_extension_abi`.

## Recommended stack

| Need | Choice |
|---|---|
| Arma extension exports | own exports (`arma-rs` rejected) |

## Hard gaps

- **Distance curve and emitter cone:** no standalone X3DAudio calculator exists; the port writes the
  piecewise curve and the cone gain itself, about 150 lines.

## Sources

[arma-rs](https://github.com/BrettMayson/arma-rs) · [windows-sys](https://crates.io/crates/windows-sys)
````
