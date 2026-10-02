**Status:** live

# Game link protocol

The localhost TCP link between an Arma extension (a Windows DLL running inside Proton) and a
TeamSpeak plugin (a native Linux `.so`). It replaces the 0.9.12 named pipe and the 1.0 shared memory,
which live inside wineserver where a native plugin cannot reach them. The game-facing behaviour of
`"task_force_radio_pipe" callExtension "<payload>"` stays byte-for-byte what the original extensions
return; only the transport is new. The original behaviour is specified in the
[extensions spec](/documentation/program/behaviour_specs/extensions.md).

Crates: `tfar_game_link_protocol` (frames and codec), `tfar_game_link_server` (the plugin side),
`tfar_game_link_client` (the extension side), `tfar_arma_extension_abi` (the `RVExtension` export).

## Roles

- The **plugin listens**; the **extension connects**. Under Proton, wine sockets are host sockets, so
  a DLL inside Arma reaches a listener on the host's loopback.
- One link per mod version. The 0.9 and 1.0 pairs use different ports and never talk to each other.
- The plugin accepts one session at a time. A new connection whose HELLO is accepted replaces the
  previous session, which is closed (Arma restarted, or a second instance took over).
- Both sides set `TCP_NODELAY`.

## Ports

The plugin binds `127.0.0.1` only, never another interface.

| Plugin | Port | With `-tfdebug` |
|---|---|---|
| 0.9 | 47810 | 47811 |
| 1.0 | 47820 | 47821 |

- **Extension:** uses the debug port when Arma's command line contains `-tfdebug`, the same test as
  the original `isDebugArmaInstance` (`reference/tfar_1_0/extensions/task_force_radio_pipe/stdafx.h`
  defines the parameter; `reference/tfar_1_0/extensions/task_force_radio_pipe/dllmain.cpp:21` picks
  the debug pipe).
- **Plugin:** listens on the normal port. The original plugin switched to the debug pipe only by
  recompiling (`reference/tfar_0_9_12/ts/src/plugin.cpp:51`). Both plugins listen on the debug port
  instead when TeamSpeak's environment sets `TFAR_GAME_LINK_DEBUG` to exactly `1`; any other value
  keeps the normal port. This is the default answer to the roadmap's debug-port question, pending
  the owner (`listen_port.rs` in each plugin app).
- **Test override:** `TFAR_GAME_LINK_PORT`, set to a port from 1 to 65535, makes the plugin listen
  on that port and wins over `TFAR_GAME_LINK_DEBUG`; any other value is ignored with a warning. It
  exists for test suites that load the plugin, so parallel tests never fight over the fixed ports
  (`apps/tfar_plugin_0_9/tests/plugin_in_mock_host.rs`). The extensions have no override: they
  always use the port of the table.
- A port already in use: the plugin logs it to the TeamSpeak log and the current tab, stays
  dormant, and retries the bind every 5 s.

## Frames

```text
offset  size  field
0       4     length   u32 little-endian: bytes after this field (kind + payload)
4       1     kind     u8, table below
5       n     payload  postcard encoding of the kind's fields, in the order listed
```

- `length` is at least 1 and at most 1,048,576. A larger or zero length, an unknown kind, or a payload
  that does not decode closes the connection; the receiver logs the reason.
- Text fields are byte strings: the call text exactly as Arma passed it (without the NUL), or the
  reply exactly as the plugin built it. Nothing is re-encoded on the wire.

| Code | Kind | Sender | Payload fields |
|---|---|---|---|
| 0x01 | `HELLO` | both | magic `[u8; 4]` = `TFAR`; protocol `u16` = 1; flavour `u8` (9 = 0.9, 10 = 1.0) |
| 0x02 | `SYNC_REQ` | extension | id `u32` (counts up from 1 per session); text bytes |
| 0x03 | `SYNC_RESP` | plugin | id `u32` (the request's id); text bytes |
| 0x04 | `ASYNC` | extension, 1.0 only | text bytes, with the trailing `~` removed |
| 0x05 | `FOCUS` | extension, 1.0 only | arma_focused `bool` |
| 0x06 | `STATUS` | plugin | needcfg `bool` (always false for 0.9) |

## Handshake

1. The extension connects and sends `HELLO` with its flavour.
2. The plugin checks the magic, the protocol number and that the flavour is its own. On any mismatch
   it closes the connection without a reply and logs the peer's values.
3. On success the plugin answers `HELLO` (its own flavour), then one `STATUS`.
4. The session is up for the extension when the plugin's `HELLO` arrives. The 1.0 extension then
   sends one `FOCUS` with the current state.

## Extension semantics

Every call starts the same way: empty input → empty output and nothing is sent, as the original
(`reference/tfar_1_0/extensions/task_force_radio_pipe/task_force_radio_pipe.cpp:9`). Replies are
written truncated to `outputSize − 1` bytes, cut back to a UTF-8 character boundary, then a NUL.

### 0.9 (`tfar_extension_0_9`): everything is sync

1. No session: connect (see Timeouts and backoff). If that fails, or a backoff is running, reply
   `Pipe not opened from TS plugin`.
2. Send `SYNC_REQ` with the whole call text (a trailing `~` has no meaning in 0.9 and is sent as is).
3. Wait at most 1000 ms for the `SYNC_RESP` with the same id, and reply its text. A response with an
   older id is discarded.
4. The plugin closed the connection during the call → `Pipe was closed by TS`. The wait timed out →
   `Not connected to TeamSpeak`.

The 0.9 extension sends no `ASYNC` and no `FOCUS`.

### 1.0 (`tfar_extension_1_0`): sync, `~` async, DFRAME, MISSIONEND, FOCUS

1. No live session (none, or no frame from the plugin for 1000 ms): try to connect; while it is not
   up, every call, sync or async, replies `Not connected to TeamSpeak`
   (`reference/tfar_1_0/extensions/task_force_radio_pipe/SharedMemoryTransfer.cpp:250`).
2. Input ending in `~`: send `ASYNC` and reply at once, without waiting for the plugin:
   - command name `DFRAME` → `NEEDCFG` when the latest `STATUS` said needcfg, else `OK`;
   - command name `MISSIONEND` → an empty reply;
   - any other async command → `OK`.

   The original tests only the first character (`D`, `M`); no other async command starts with
   either letter, so matching the command name answers every real call identically.
3. Any other input: `SYNC_REQ`, wait at most 1000 ms for the matching `SYNC_RESP`, reply its text;
   on timeout an empty reply, as the original. A session that closes while the call waits (the
   plugin dropped the connection, or the session died) replies `Not connected to TeamSpeak`, the
   same as a call with no live session; 1.0 has no `Pipe was closed by TS`.
4. Focus: on every call, at most every 250 ms, the extension reads the foreground window
   (`GetForegroundWindow`, `GetClassNameW`) and sends `FOCUS` when "is Arma in front" changed. The
   class-name test is the original's (`reference/tfar_1_0/ts/src/Teamspeak.cpp:207`).

## Plugin semantics

- **Sync:** each `SYNC_REQ` is handled on the link thread and answered with one `SYNC_RESP`. The
  command crate builds the text (`PONG`, `OK`, `SPEAKING`, the 1.0 two-character codes and so on). A
  sync command may also queue follow-up work on the async queue (1.0 POS does).
- **Async (1.0):** `ASYNC` frames go to one FIFO queue served by one worker thread, strictly in
  arrival order (F-007 fixes the original's LIFO). The command name `MISSIONEND` empties the
  unprocessed queue and zeroes the last-game-message time, the same effect as the original's
  `lastGameTick = 0` (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:415`).
- **STATUS:** sent after `HELLO`, every 500 ms as a heartbeat, and at once when needcfg changes.
  needcfg starts true, turns false on any SETCFG, and turns true again when the game disconnects.
- **FOCUS (1.0):** the latest value gates the serious-channel move (`moveWhileTabbedOut`). Before the
  first `FOCUS` of a session, Arma counts as not in front.

### Connected and disconnected

| | 0.9 plugin | 1.0 plugin |
|---|---|---|
| Link connected | A session is up | A session is up |
| Game connected (in game) | A command other than VERSION arrived less than 4000 ms ago (F-022: no serious-channel condition) | A `SYNC_REQ` or `ASYNC` arrived less than `pluginTimeout` seconds ago (setting, default 4) and no MISSIONEND since |
| Transitions | Checked by the service thread every 100 ms | Debounced to at most one connect or disconnect per 500 ms |
| On connect | play the `on` sound, `onGameStart` (serious channel move), data frame 0 | play the `on` sound, `onGameStart`, data frame 0 |
| On disconnect | unmute, play the `off` sound, alive false, data frame invalid, `onGameEnd` | unmute all, play the `off` sound, `onGameEnd`, needcfg true, alive false, data frame invalid |

`HELLO` and `FOCUS` never count as game messages. A dropped session makes the link disconnected at
once; the game counts as disconnected once its timeout passes.

## Timeouts and backoff

| What | Value |
|---|---|
| TCP connect to the plugin port | ≤ 50 ms |
| Plugin `HELLO` after connecting | ≤ 250 ms, else the connection is dropped |
| Backoff after a failed connect or a dropped session | 1 s: calls in that second reply at once without a new attempt |
| Sync wait for the matching `SYNC_RESP` | ≤ 1000 ms |
| Session dead without any plugin frame | 1000 ms (the heartbeat is 500 ms) |
| Socket write | ≤ 1000 ms, else the session is dropped |

The extension never blocks Arma longer than about 1.3 s on one call (connect, HELLO and the sync
wait), and in steady state never longer than the 1000 ms sync wait.

## Game-facing strings

These are produced by the extension itself and are byte-identical to the originals.

| String | 0.9 | 1.0 |
|---|---|---|
| `Pipe not opened from TS plugin` | no session and connect failed or backoff running | — |
| `Pipe was closed by TS` | the plugin closed the connection during a call | — |
| `Not connected to TeamSpeak` | sync wait timed out | no live session, for every call; a session closed during a sync call |
| `OK` | — | any async call except DFRAME with needcfg and MISSIONEND |
| `NEEDCFG` | — | `DFRAME~` while the latest STATUS says needcfg |
| empty | empty input | empty input, `MISSIONEND~`, sync wait timed out |

The original 0.9 `Pipe error <n>` and the 1.0 `TFAR SHAMEM Too big request` message box have no
cause left in the new transport (no Win32 pipe errors, no 2044-byte slot), so they are never
produced. Every other reply (`PONG`, `FAIL`, `DONE`, `SPEAKING`, `NOT_SPEAKING`,
`UNKNOWN COMMAND`, `ASYNC COMMAND SENT IN SYNC CONTEXT`, `ERROR_GETTING_SERVER_NAME`, the 1.0
two-character speaking codes) comes from the plugin's command crates, as the behaviour specs list.

## Ordering

- Frames on one session arrive in send order (one TCP stream).
- Async commands run first-in, first-out on one worker.
- A sync request is answered on the link thread as it arrives, so it can overtake async commands
  still queued; the original also served the sync slot first.
- After a MISSIONEND, queued async commands of the ended mission never run.

## Dormant plugin

A plugin whose game link has never had a game message this TeamSpeak session (0.9: never in game;
1.0: never game-connected) is **dormant**. A dormant plugin:
- writes no metadata, nickname or channel, and sends no mute, unmute or plugin command;
- passes every audio callback through untouched;
- keeps listening on its port and answers the handshake.

This lets both Rust plugins be installed at once: only the version whose mod is running acts.
Switching mods without restarting TeamSpeak leaves the first plugin active but out of game; the
open findings in [progress](/documentation/program/progress.md) track that case (OF-011).

## Sequence

```text
Arma (1.0 mod)            tfar_extension_1_0              tfar_plugin_1_0 (127.0.0.1:47820)
     |                            |                                  |
     |  "TS_INFO\tPING"           |                                  |
     |--------------------------->|  connect (<= 50 ms)              |
     |                            |--------------------------------->|  accept
     |                            |  HELLO {TFAR, 1, 10}             |
     |                            |--------------------------------->|  check magic, protocol, flavour
     |                            |  HELLO {TFAR, 1, 10}, STATUS{1}  |
     |                            |<---------------------------------|
     |                            |  FOCUS {true}                    |
     |                            |--------------------------------->|
     |                            |  SYNC_REQ {1, "TS_INFO\tPING"}   |
     |                            |--------------------------------->|  link thread answers
     |                            |  SYNC_RESP {1, "PONG"}           |
     |  "PONG"                    |<---------------------------------|
     |<---------------------------|                                  |
     |  "DFRAME~"                 |  ASYNC {"DFRAME"}                |
     |--------------------------->|--------------------------------->|  FIFO queue → worker; game connected
     |  "NEEDCFG"                 |                                  |
     |<---------------------------|  STATUS {needcfg = 1} (cached)   |
     |  "SETCFG\t...~"            |  ASYNC {"SETCFG\t..."}           |
     |--------------------------->|--------------------------------->|  worker applies; needcfg = 0
     |  "OK"                      |  STATUS {0}                      |
     |<---------------------------|<---------------------------------|
     |          ...               |  STATUS every 500 ms             |
     |  "MISSIONEND~"             |  ASYNC {"MISSIONEND"}            |
     |--------------------------->|--------------------------------->|  queue emptied, last game time 0
     |  ""                        |                                  |  -> game disconnected (debounced)
     |<---------------------------|                                  |
```
