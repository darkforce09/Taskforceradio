**Status:** live

# Architecture of the Rust port

How the port is built: the workspace and its crate categories, the localhost link between the Arma
extensions and the TeamSpeak plugins, and the voice and radio audio chain of each version. Read
these before changing a crate boundary, the link, or the sound.

## Contents

```text
documentation/architecture/
├── audio_pipeline.md       the voice and radio chain per version, the crates and libraries per step
├── game_link_protocol.md   ports, frames, handshake, per-version semantics, timeouts, game-facing strings
└── workspace_layout.md     the tree, the crate categories, where a new crate goes, the dependency diagram
```

## How it works

Each document describes the design as it will be built, in the present tense; the original C++
behaviour it replaces is in the behaviour specs, and each deliberate difference names its bug-fix
id. A change to the design changes the document in the same commit as the code.

## Code

- The crates these documents describe (planned: crates/game_link/, crates/audio/,
  crates/version_0_9/, crates/version_1_0/ and the four apps under apps/).

## Boundaries

- Depends on: the program plan's decisions, the behaviour specs, and the library survey.
- Used by: every agent brief from M1 on, the crate READMEs, and the roadmap.
- Rules: present tense, no history; diagrams in `text` blocks; frame layouts and game-facing strings
  change only with a decision.

## Related documentation

- [Program plan](/documentation/program/program_plan.md) — decisions D1–D22 behind the design.
- [Laws and gates](/documentation/program/laws_and_gates.md) — the edge matrix and firewalls that
  keep the layout.
- [Bug fixes](/documentation/program/bug_fixes.md) — the F-ids the documents cite.
