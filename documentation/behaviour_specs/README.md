# Behaviour specs of the original TFAR code

Frozen functional specs of the original C++ TeamSpeak plugins and Arma extensions of both TFAR
versions, written from the reference trees with file:line evidence, bugs included. Read the spec of
the code a crate replaces before changing its behaviour.

- [0.9 plugin](/documentation/behaviour_specs/tfar_0_9_plugin.md)
- [1.0 plugin](/documentation/behaviour_specs/tfar_1_0_plugin.md)
- [Extensions](/documentation/behaviour_specs/extensions.md)
- [Windows dependencies](/documentation/behaviour_specs/windows_dependencies.md)

Each spec describes the original as it was read and is never edited. A bug the port fixes goes into
[bug fixes](/documentation/architecture/bug_fixes.md). File:line references are relative to the
source folder each spec names at its top (for example `reference/tfar_1_0/ts/src/`).
