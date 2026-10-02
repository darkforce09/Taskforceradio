**Status:** live

# Template: decision entry

**When to use:** one decision in the project's [decision log](/documentation/decisions/README.md): a
choice that shapes the port and would otherwise be argued again. An entry says what was decided, why,
what follows from it, and which earlier entry it replaces. Questions not yet decided stay in the
roadmap's open questions table until they are settled. The
[documentation standards](/documentation/standards/documentation_standards.md) fix the entry's parts.

## Skeleton

Copy the block and replace every `<…>` placeholder; each one says what goes there. Each entry is a `###`
heading at the end of the log's file for its subject, with its four parts in this order.

````markdown
### <YYYY-MM-DD> — <the decision, as a short statement; a plan id in brackets when it has one>

**Context:** <the problem, the constraints, and the options on the table when it was decided>

**Decision:** <what was decided, precisely enough to check the code against it>

**Consequences:** <what follows: what the code must do, what it rules out, the costs accepted, and the
gate or test that holds the decision in place>

**Supersedes:** <the date and heading of the entry this one replaces, as that heading spells them; or
"none.">
````

The date is the day the decision was taken. An entry is never reworded once written: when a decision
changes, a new entry records the new one and names the old entry under Supersedes, so the log shows which
decision holds. Context describes the situation at that date, so the log is the one live document that
may tell how things changed. A heading's anchor is its text in lowercase, with the spaces turned into
hyphens and the dash and brackets dropped, which is how other documents link an entry:

```text
[Panics abort](/documentation/decisions/workspace.md#2026-10-02--panics-abort-and-no-panic-path-exists-d12)
```

## Worked sample

The game link transport entry of the decision log, shortened. The sample sits in a fenced block, so no
gate reads it.

````markdown
### 2026-10-02 — The game link is length-prefixed frames over localhost TCP

**Context:** Both original mods link Arma and TeamSpeak through Win32 named objects: 0.9.12 through the
named pipe `\\.\pipe\task_force_radio_pipe`, 1.0 through the shared memory `Local\TFARSHAMEM` with events
and a mutex. Under Proton these objects live inside wineserver, where a native Linux TeamSpeak plugin
cannot reach them. The Flatpak sandbox shares the host network.

**Decision:** The TeamSpeak plugin listens on 127.0.0.1; the extension connects to it. Frames are
`u32 LE length | u8 kind | payload`, and a connection opens with `HELLO{magic "TFAR", protocol, flavour}`.

**Consequences:** The extension needs no Win32 object but its socket; the plugin needs no Windows code.
The game-facing reply strings stay exactly the originals. The frame codec has round-trip tests, and the
probe proves the link under wine.

**Supersedes:** none.
````
