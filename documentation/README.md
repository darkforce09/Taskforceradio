# Documentation

Every document of Taskforceradio. Code READMEs (one per crate and category under `crates/` and
`tools/`) say what a crate does and where it stops; the documents here explain what spans crates.

| Area | Holds |
|---|---|
| [architecture/](/documentation/architecture/README.md) | The game link protocol, the audio pipeline and the [bug fixes](/documentation/architecture/bug_fixes.md) catalogue |
| [behaviour_specs/](/documentation/behaviour_specs/README.md) | What the original plugins and extensions do, cited by `reference/` file and line |
| [decisions/](/documentation/decisions/README.md) | The dated decision log, one file per subject |
| [runbooks/](/documentation/runbooks/README.md) | Manual install and live test |
| [standards/](/documentation/standards/README.md) | Code and documentation, testing, crate boundary rules |
| [glossary.md](/documentation/glossary.md) | TFAR, TeamSpeak, audio and project terms |

The laws, environment and commands are in [CLAUDE.md](/CLAUDE.md); open work is the checklist in
the root [README](/README.md#open-work).

When two sources disagree, the higher one wins and the lower one is fixed in the same commit:
running code, then CLAUDE.md, then decisions, then standards, then the other documents. The original
C++ in `reference/` is the specification of behaviour, not of design: where the
[bug policy](/documentation/decisions/bug_policy.md) fixes a bug, the fix wins over the original.
The behaviour specs are frozen records of the originals as they were read; new facts go into a new
record.
