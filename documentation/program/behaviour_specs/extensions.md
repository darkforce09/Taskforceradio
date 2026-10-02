**Status:** frozen record (2026-10-02)

# Arma extensions (`task_force_radio_pipe`): behaviour spec

This describes both original C++ extensions. Every SQF call uses the plain-string form `"task_force_radio_pipe"
callExtension "<payload>"` with TAB-separated fields, never the array form. Arma x64 loads
`task_force_radio_pipe_x64.dll` from the mod root. The Rust ports keep the extension name and every reply
string. Only the transport changes (see `/documentation/architecture/game_link_protocol.md`).

## 1. 0.9.12 (`reference/tfar_0_9_12/arma3/@task_force_radio/task_force_radio_pipe/`)
- `task_force_radio_pipe.cpp` (89 lines). The DLL **creates** the named pipe server
  `\\.\pipe\task_force_radio_pipe` (debug variant `\\.\pipe\task_force_radio_pipe_debug` when the command
  line contains `-tfdebug`) with `CreateNamedPipe(PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED,
  PIPE_TYPE_MESSAGE, 1 instance)` and a NULL DACL.
- `RVExtension(output, outputSize, input)`: empty input → return. `SetNamedPipeHandleState(message mode)`,
  then `TransactNamedPipe(input → output)`, which is a blocking request/response for **every** call.
- Errors (the pipe is reopened unless `ERROR_PIPE_LISTENING`):

  | Win32 error | Output string |
  |---|---|
  | `ERROR_NO_DATA` | `Pipe was closed by TS` |
  | `ERROR_PIPE_LISTENING` | `Pipe not opened from TS plugin` |
  | 230 | `Not connected to TeamSpeak` |
  | other | `Pipe error <n>` (buggy `sprintf_s` length) |

- SQF callers (`reference/tfar_0_9_12/arma3/@task_force_radio/addons/task_force_radio/functions/`):
  `fn_sessionTracker.sqf:27` (TRACK), `fn_getTeamSpeakChannelName.sqf:19`, `fn_isSpeaking.sqf:21` (expects
  `SPEAKING`), `fn_sendPlayerKilled.sqf:22`, `fn_processPlayerPositions.sqf:104`, `fn_ProcessTangent.sqf:34`,
  `fn_isTeamSpeakPluginEnabled.sqf:19` (expects `PONG`), `fn_sendFrequencyInfo.sqf:101`.

## 2. 1.0 (`reference/tfar_1_0/extensions/task_force_radio_pipe/`)
- Files: `dllmain.cpp` (DllMain + a `main()` speed test calling POS 10,000×), `task_force_radio_pipe.cpp`
  (`RVExtension` → `transfer.transactMessage`, then `output[outputSize-1] = 0`), `SharedMemoryTransfer.*`
  (active), `NamedPipeTransfer.*` (unused), `stdafx.h` (`PIPE_TIMEOUT 1000`, `-tfdebug` parameter).
- The DLL **opens** the plugin-created objects: `Local\TFARSHAMEM_EVTREQ`, `Local\TFARSHAMEM_EVTRESP`,
  `Local\TFARSHAMEM_MTX`, mapping `Local\TFARSHAMEM` (`SharedMemoryTransfer.cpp:150-240`). The layout is in
  `/documentation/program/behaviour_specs/tfar_1_0_plugin.md` §2.1.
- `transactMessage` (`SharedMemoryTransfer.cpp:250-281`):
  1. Not ready → the stored error message, else `Not connected to TeamSpeak`.
  2. `!isConnected()` (plugin tick older than 1000 ms) → `Not connected to TeamSpeak`.
  3. Input ends in `~` → `doAsyncRequest`. Then `D…` (DFRAME) with `needsConfigRefresh` → `NEEDCFG`;
     `M…` (MISSIONEND) → `shutdown()` (game tick = 0, queue emptied) and an empty reply; otherwise `OK`.
  4. Else `doSyncRequest`: write sync slot, `SetEvent(REQ)`, `SignalObjectAndWait(REQ, RESP, 1000)`, read the
     answer (empty on timeout).
- A request longer than 2044 bytes shows a MessageBox (`TFAR SHAMEM Too big request`) and is dropped.
- SQF callers (`reference/tfar_1_0/addons/`):
  - sync: `core/functions/fnc_processPlayerPositions.sqf:61` (POS / IS_SPEAKING_BULK),
    `plugin/fnc_getTeamSpeakChannelID.sqf:21`, `plugin/fnc_isSpeaking.sqf:23` (2-char reply),
    `plugin/fnc_getTeamSpeakChannelName.sqf:21`, `plugin/fnc_getTeamSpeakServerUID.sqf:21`,
    `plugin/fnc_isTeamSpeakPluginEnabled.sqf:21` (`PONG`), `plugin/fnc_getTeamSpeakServerName.sqf:21`,
    `plugin/fnc_getTeamspeakPluginVersion.sqf:21`, `plugin/fnc_sendPlayerInfo.sqf:26`,
    `plugin/fnc_sendFrequencyInfo.sqf:132`, `plugin/fnc_sendPlayerKilled.sqf:26`,
    `plugin/fnc_betaTracker.sqf:45`, `plugin/fnc_sessionTracker.sqf:48`.
  - async (`~`): `plugin/fnc_pluginNextDataFrame.sqf:22` (`DFRAME~`, reacts to `NEEDCFG`),
    `plugin/fnc_sendSpeakerRadios.sqf:118`, `plugin/fnc_processTangent.sqf:36`,
    `plugin/fnc_setPluginSetting.sqf:28` (SETCFG), `plugin/fnc_releaseAllTangents.sqf:31`,
    `events/fnc_onMissionEnd.sqf:21` (`MISSIONEND~`), `antennas/functions/fnc_pluginAddRadioTower.sqf:35`,
    `antennas/functions/fnc_pluginRemoveRadioTower.sqf:26`, `core/functions/fnc_initKeybinds.sqf:206`
    (`collectDebugInfo~`).

## 3. Arma ABI facts
- x64 export: `void RVExtension(char* output, int outputSize, const char* function)`, undecorated.
  `RVExtensionVersion(char* output, int outputSize)` is optional. `outputSize` includes the NUL.
- `arma-rs` cannot serve this call form (it looks the whole string up as a command name), so the exports are
  written by hand.
- BattlEye blocks extensions that are not whitelisted, so a new DLL needs BE off or a non-BE server.
