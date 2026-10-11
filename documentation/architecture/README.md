# Architecture

How the port works across crates: the localhost link between the Arma extensions and the TeamSpeak
plugins, the voice and radio audio chain of each version, and the original bugs the port fixes on
purpose. Read these before changing the link or the sound.

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — frames, ports, flavours
  and the replies Arma sees.
- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — the voice and radio chain of
  each version.
- [Bug fixes](/documentation/architecture/bug_fixes.md) — every original bug fixed on purpose, with
  the crate and test that hold the fix.

The documents describe the code as it is, in the present tense; the original behaviour is in the
[behaviour specs](/documentation/behaviour_specs/README.md). A design change updates its document in
the same commit. Frame layouts and game-facing strings change only with a decision.
