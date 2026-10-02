**Status:** live

# Behaviour specs of the original TFAR code

Frozen functional specs of the original C++ TeamSpeak plugins and Arma extensions of both TFAR
versions, written from the reference trees with file:line evidence, bugs included. Agents porting a
crate read the spec of the code it replaces; the fixed bugs are catalogued separately.

## Contents

```text
documentation/program/behaviour_specs/
├── extensions.md             both task_force_radio_pipe extensions: transport, replies, SQF callers, ABI
├── tfar_0_9_plugin.md        the 0.9.12 plugin: state, threads, pipe commands, audio, TS API, sqlite
├── tfar_1_0_plugin.md        the 1.0 plugin: modules, shared memory, commands, audio, settings, debug
└── windows_dependencies.md   every Windows-only dependency of the plugins and its Rust replacement
```

## How it works

Each spec is a frozen record dated 2026-10-02: it describes the original behaviour and is never
edited. A correction goes into the open findings of
[progress](/documentation/program/progress.md), and a bug into
[bug fixes](/documentation/program/bug_fixes.md). File:line references are relative to the source
folder each spec names at its top (for example `reference/tfar_1_0/ts/src/`).

## Code

- The original sources under `reference/tfar_0_9_12/` and `reference/tfar_1_0/`, which the specs
  describe.

## Boundaries

- Depends on: the reference trees, read-only.
- Used by: the crate catalogue, the bug fixes, the architecture documents and every porting agent.
- Rules: frozen; new findings never edit a spec.

## Related documentation

- [Bug fixes](/documentation/program/bug_fixes.md) — what the port changes on purpose.
- [Crate catalogue](/documentation/program/crate_catalogue.md) — which crate ports which section.
