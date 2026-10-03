**Status:** live

# Game link decisions

Where the parts run, how the Arma extension and the TeamSpeak plugin talk, on which ports, how window
focus crosses over, and which bindings the plugin and the extension are built on. The full protocol is in
the [game link protocol](/documentation/architecture/game_link_protocol.md).

### 2026-10-02 — Proton only: Windows extensions, Linux plugins (D2)

**Context:** The owner runs Arma 3 as the Windows build under Proton and TeamSpeak 3.6.2 as the native
Linux Flatpak (plugin API 26). A native Linux Arma build or a Windows TeamSpeak is not part of the setup,
and 32-bit Arma is not either.

**Decision:** Proton only. The extensions target `x86_64-pc-windows-gnu`; the plugins target Linux
`x86_64` only.

**Consequences:** The extensions are cross-compiled with the mingw linker and `+crt-static`, so the DLL
needs nothing beside it; the windows clippy lane of `cargo xtask ci ci-local` checks them. The plugins
contain no Windows code, and Win32 calls (`GetForegroundWindow`, `MessageBox`, `%APPDATA%`) are replaced
(see the focus entry below and the behaviour specs).

**Supersedes:** none.

### 2026-10-02 — The TeamSpeak bindings are ts3plugin-sys, the export layers are ours (D22)

**Context:** The TeamSpeak plugin SDK is a C API: the plugin exports `ts3plugin_*` functions and calls
back through a function table. The high-level `ts3plugin` wrapper holds a global mutex with `unwrap` and
lacks callbacks TFAR needs. On the Arma side, `arma-rs` looks the whole input string up as a command
name, so it cannot take TFAR's plain-string, tab-separated calls.

**Decision:** `ts3plugin-sys` 0.5 (API 26) provides the TeamSpeak bindings; the export layer is ours, in
`tfar_ts3_host`. `arma-rs` is rejected; the `RVExtension` exports are ours, in
`tfar_arma_extension_abi`.

**Consequences:** `tfar_ts3_host` writes every `ts3plugin_*` export, including the rolloff, self-variable,
`infoData` and `freeMemory` exports, owns the C-string rules, and keeps the audio and GUI threads apart.
Its struct layouts are checked against the official API 26 headers with `size_of` and `offset_of` tests.
`tfar_arma_extension_abi` writes `RVExtension` and `RVExtensionVersion` with truncating output writes.

**Supersedes:** none.

### 2026-10-02 — The game link is length-prefixed frames over localhost TCP

**Context:** Both original mods link Arma and TeamSpeak through Win32 named objects: 0.9.12 through the
named pipe `\\.\pipe\task_force_radio_pipe`, 1.0 through the shared memory `Local\TFARSHAMEM` with events
and a mutex. Under Proton these objects live inside wineserver, where a native Linux TeamSpeak plugin
cannot reach them. The TeamSpeak Flatpak sandbox shares the host network, and Proton's wine has working
sockets.

**Decision:** The TeamSpeak plugin listens on 127.0.0.1 and the extension connects to it. Frames are
`u32 LE length | u8 kind | payload`, and a connection opens with `HELLO{magic "TFAR", protocol,
flavour}`. The extension sends `SYNC_REQ`, `ASYNC` and `FOCUS`; the plugin sends `SYNC_RESP` and
`STATUS{needcfg}`. The extension connects lazily, with a connect timeout of at most 50 ms and a 1 s
backoff; a sync call waits at most 1000 ms. The game-facing reply strings are exactly the originals:
`Not connected to TeamSpeak`, `Pipe not opened from TS plugin`, `OK`, `NEEDCFG`. In 1.0, `MISSIONEND`
zeroes the plugin's last-game-message time as the original does, and the async queue is FIFO.

**Consequences:** Neither side needs a Win32 named object. The SQF sees the same strings as before, so
the mods run unchanged. A plugin whose game link has never connected in this TeamSpeak session stays
[dormant](/documentation/glossary.md#dormant-plugin), so the two plugins can be loaded at once. The frame
codec has round-trip tests in `tfar_game_link_protocol`, and the extension probe proves the link under
wine.

**Supersedes:** none.

### 2026-10-02 — Each plugin has its own port, and -tfdebug adds one

**Context:** The 0.9.12 and 1.0 plugins may be loaded in TeamSpeak at the same time, and each extension
must reach only its own plugin. The originals used a separate debug pipe or object name when Arma ran
with `-tfdebug`.

**Decision:** The plugins listen on fixed ports of 127.0.0.1:

| Plugin | Port | With `-tfdebug` |
|---|---|---|
| 0.9 | 47810 | 47811 |
| 1.0 | 47820 | 47821 |

**Consequences:** Each extension reaches only its own mod's plugin; the `HELLO` flavour names the mod as
well. The ports are constants in `tfar_game_link_protocol`, the one place both sides read them from.

**Supersedes:** none.

### 2026-10-02 — Window focus travels from the extension in a FOCUS frame

**Context:** The 1.0 plugin calls `GetForegroundWindow` to tell whether Arma has focus, for the
`moveWhileTabbedOut` setting. A native Linux plugin cannot see the Proton window this way; the extension,
inside the game process under wine, can.

**Decision:** `GetForegroundWindow` belongs in the extension, which reports the game window's focus in a
`FOCUS` frame; the 1.0 plugin uses the last focus it received for `moveWhileTabbedOut`.

**Consequences:** The focus query lives in `tfar_arma_extension_abi` (the one crate allowed
`windows-sys`); the 0.9 extension sends no `FOCUS` frames, since 0.9.12 has no focus-aware behaviour.

**Supersedes:** none.

### 2026-10-03 — Each plugin listens on both its normal and its -tfdebug port (D24)

**Context:** An Arma instance started with `-tfdebug` connects to the debug port of its mod (47811 or
47821), the normal one to the normal port. The original plugins switched to the debug pipe only by
recompiling. The plugin apps listened on one port, the debug one only when TeamSpeak's environment set
`TFAR_GAME_LINK_DEBUG=1`, a setting outside the game that a player has to change by hand; that variable
was the game link protocol's default, pending the owner, and never an entry here.

**Decision:** Each plugin binds both its normal and its `-tfdebug` port on 127.0.0.1. Each port has its
own accept loop and its own bind retry, and both feed one session slot: the newest connection whose
`HELLO` is valid wins and replaces the session before it. `TFAR_GAME_LINK_DEBUG` is removed. The
test-only `TFAR_GAME_LINK_PORT` stays, so test suites that load a plugin never fight over the fixed
ports.

**Consequences:** A normal and a debug Arma instance need no plugin change; the one that connects last
holds the session. A port already in use fails alone and is retried while the other port keeps working.
`tfar_game_link_server` takes a list of ports in its `ServerConfig` and exposes the bound addresses;
its tests cover a `HELLO` on each port, the session switching ports, and a busy debug port.

**Supersedes:** none.

### 2026-10-03 — A plugin returns to dormant after its game-end handling (D25)

**Context:** A plugin stays dormant until its first game message of the TeamSpeak session, then acts.
Under that rule it never went back to sleep, so after playing 0.9 and then 1.0 in one TeamSpeak session
the 0.9 plugin kept writing its out-of-game metadata beside the 1.0 one (OF-011). The options were to
accept it and restart TeamSpeak between mods, or to let a plugin sleep again after its game ends.

**Decision:** Once a plugin's game-end handling has finished (every mute lifted, the off sound played,
the channel and nickname given back, the metadata set to "not in game" once), the plugin is dormant
again: no metadata, nickname, channel, mute or plugin-command side effect until game activity resumes.
Its next game message wakes it as the first one did.

**Consequences:** Both Rust plugins can stay installed and the mods can be switched within one TeamSpeak
session; only the plugin whose mod is in a game acts. The game link stays bound and answers the
handshake while dormant. Each plugin app holds a mock test that sees no side effect for 10 s of service
ticks after a game end, and side effects again once game activity resumes.

**Supersedes:** the dormant rule of "2026-10-02 — The game link is length-prefixed frames over localhost
TCP", under which a plugin is dormant only until its game link first connects.
