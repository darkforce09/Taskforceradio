# Decisions

The log of choices that shape the port and would otherwise be argued again, one file per subject:

- [Workspace](/documentation/decisions/workspace.md) — layout, conventions, the library map.
- [Game link](/documentation/decisions/game_link.md) — the TCP link, ports, dormancy, the focus frame.
- [Audio](/documentation/decisions/audio.md) — the audio libraries, HRTF, the audio-thread locking.
- [Bug policy](/documentation/decisions/bug_policy.md) — what happens to the originals' bugs and
  features.

An entry is a `### YYYY-MM-DD — <the decision>` heading with **Context**, **Decision**,
**Consequences** and **Supersedes**, added at the end of its subject's file. The ids in the headings
(D1–D27) are the ones the entries were recorded under. An entry is never reworded: a changed decision
gets a new entry that names the old one under Supersedes, so the latest entry on a subject holds.
Questions not yet decided live in the [open-work checklist](/README.md#open-work).
