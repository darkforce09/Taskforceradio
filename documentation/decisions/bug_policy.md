**Status:** live

# Bug policy decisions

What the port does with the originals' bugs and features, where C and C++ may still appear, and in which
order the work is done. The bugs themselves are catalogued in
[bug fixes](/documentation/program/bug_fixes.md).

### 2026-10-02 — Fix every bug; wire formats stay compatible (D3)

**Context:** Reading both plugins for the behaviour specs found dozens of bugs: an async queue read
last-in first-out, data races, effect pointers used after their lock is released, unchecked token
indexing, gain that wraps on overflow, a disabled HRTF, wrong error-level slopes and signs, fall-throughs
to `FAIL`, and more. Players on Windows keep running the original plugins and talk to Linux players over
the same TeamSpeak server.

**Decision:** Fix all bugs in the catalogue. Sound may differ from what Windows players hear. Wire
formats stay compatible: plugin commands, metadata and the game-facing reply strings are byte-compatible
with the originals.

**Consequences:** Every fix has an F-id in the bug fixes record, its owning crate and a test. A fix that
would change a plugin command, the metadata block or a reply string is not a fix but a format change, and
needs its own decision.

**Supersedes:** none.

### 2026-10-02 — Keep every feature (D4)

**Context:** Some original features are rarely used or need services on the internet: the update check,
Piwik tracking (`TRACK`), the debug window, Tracy, the API profiler, the `/tfar` commands and
`collectDebugInfo`.

**Decision:** Keep every one of them: update check, Piwik `TRACK`, debug window, Tracy, API profiler,
`/tfar` commands, `collectDebugInfo`.

**Consequences:** `tfar_web_requests` covers the update check and Piwik, `tfar_debug_window` the window,
the plugin apps' `tracy` and `api_profiler` features the profiling. Windows-only parts are replaced, not
dropped: `MessageBox` becomes `printMessageToCurrentTab` plus `logMessage`, and `%APPDATA%` paths become
TeamSpeak's `getConfigPath()`.

**Supersedes:** none.

### 2026-10-02 — C and C++ only as golden-vector oracles (D9)

**Context:** The audio crates replace DSPFilters, SimpleComp, clunk and X3DAudio. Their output must be
compared with the originals', and the originals exist only as C and C++.

**Decision:** C/C++ oracles are allowed only in the tfar_golden_vectors tool, built with `cc`. They compile
DSPFilters, SimpleComp and clunk straight from `reference/`, plus a vendored FAudio `F3DAudio.c` (zlib)
in its oracle/ folder (tools/tfar_golden_vectors/oracle/).

**Consequences:** No product crate links C or C++. `cargo xtask verify language-bans` allows C and C++
sources only under the golden-vector tool's oracle/ and shim/ folders; `cc` is firewalled to that crate.

**Supersedes:** none.

### 2026-10-02 — Port order: foundations, then 0.9, then 1.0 (D10)

**Context:** The two versions share most primitives; 0.9.12 is the smaller plugin, with sync calls only
and no antennas or settings.

**Decision:** Port the foundations first, then the 0.9 track, then the 1.0 track.

**Consequences:** The milestones run foundations, the TeamSpeak host layer, the game link, the audio
primitives, services and diagnostics, then the 0.9 track and the 1.0 track, then release readiness, as
the [roadmap](/documentation/roadmap.md) sets out.

**Supersedes:** none.
