**Status:** frozen record (2026-10-02)

# TFAR 1.0 TeamSpeak plugin: behaviour spec

This is the functional spec of the C++ plugin at `reference/tfar_1_0/ts/src/`. All file:line references
are relative to that folder unless written as `extensions/...` or `addons/...` (relative to
`reference/tfar_1_0/`). It describes the original behaviour, bugs included. The Rust port fixes the bugs
listed in `/documentation/program/bug_fixes.md`.

Behaviours most likely to break a faithful reading:
- `Vector3D::normalized()` uses the wrong formula (§8).
- Subtypes are matched by string length only (§8).
- Async messages are read last-in, first-out (§2.1).
- The clunk HRTF filter is disabled, so clunk only adds a time delay (§3.4).
- X3DAudio is Windows-only (§3.3).
- Gain multiplication wraps instead of saturating (§3.7).

## 1. Module map

| File | LoC | Responsibility / key types |
|---|---|---|
| plugin.cpp | 1100 | `ts3plugin_init`/`shutdown`, PipeThread, ServiceThread, `processVoiceData` (the whole voice DSP), voice callbacks, plugin-command receive (`processPluginCommand`, `processTangentPress`), `setGameClientMuteStatus`, `isPluginEnabledForUser`, sqlite PTT-delay read |
| Teamspeak.cpp/.hpp | 1039/110 | Static wrapper over `ts3Functions`. `TeamspeakServerData` per server: muted set, my ID, original nickname and channel, channel name→ID cache. Event fan-out. All `ts3plugin_*` metadata exports and `processCommand` (`/tfar`) |
| CommandProcessor.cpp/.hpp | 667/66 | Parses game messages; sync replies; async queue plus worker thread; tangent PTT logic |
| task_force_radio.cpp/.hpp | 391/183 | `TFAR` singleton: Signals, `gameData` (local player state), lazy singletons, update check, Piwik `TRACK`, serious-mode detection, `Version` compare |
| serverData.cpp/.hpp | 234/111 | `serverDataDirectory` (serverID→`serverData`); `serverData` = vector of (hash(nick), clientID, `shared_ptr<clientData>`) sorted by hash |
| clientData.cpp/.hpp | 433/373 | Per-remote-client state, `isOverRadio`/`isOverLocalRadio`, `clientDataEffects` (lazy per-key filters and effects), circular debug log |
| RadioEffect.hpp | 296 | Radio effect classes, `processRadioEffect`, `processCompressor` |
| helpers.cpp/.hpp | 354/202 | Parsing, split, `volumeAttenuation`, `volumeMultiplifier`, `processFilterStereo`, the two `applyILD`s (simple and X3DAudio), SSE gain |
| datatypes.cpp/.hpp | 235/224 | `TeamspeakID`, `Vector3D`/`Position3D`/`Direction3D`, angles, `NetID` |
| SampleBuffer.cpp/.hpp | 37/208 | Shared-ptr view or owned buffer of interleaved i16; gain, mix, copy |
| PlaybackHandler.cpp/.hpp | 518/341 | WAV cache, playback objects, mixing into the mixed-playback callback, `SoundDirectory` paths |
| Clunk.h | 69 | Wrapper around `clunk::Hrtf` (in effect only an interaural time difference, §3.4) |
| antennaManager.cpp/.h | 69/134 | Radio towers, `findConnection` |
| SharedMemoryHandler.cpp/.hpp | 316/129 | IPC with the Arma extension |
| settings.hpp | 220 | Config keys, defaults, typed values |
| Locks.hpp, SignalSlot.hpp, Logger.*, enum.hpp | 167/62/189/193 | Infrastructure |
| pipe_handler.* | 122 | Named-pipe transport, compiled but unused (`#define USE_SHAREDMEM`, plugin.cpp:165) |
| server_radio_data.* | 196 | Dead: not compiled |
| DebugUI.*, TextRenderer.*, Shader.h, ProfilerTracy.*, profilers.*, debugExecutable.cpp | ~1.9k | Debug and profiling (§7) |
| apicompat.asm | 50 | Exports `ts3plugin_onPluginCommandEvent`, jumping to the Old or New handler |

**Dependencies.** plugin.cpp → TFAR, CommandProcessor, serverData/clientData, PlaybackHandler, RadioEffect,
helpers, Teamspeak, SharedMemoryHandler. CommandProcessor → TFAR, serverData, PlaybackHandler,
AntennaManager, Teamspeak. clientData → RadioEffect, Clunk, SimpleComp, AntennaManager, TFAR.

**Threads.**
- **PipeThread** (plugin.cpp:166, started in init :335). Loop: `isConnected()` (raises connect/disconnect
  events); write `config.needsRefresh()` into shared memory; `getData(cmd, 20ms)`; a message ending in `~`
  is stripped and queued, otherwise `processCommand` and `sendData(reply)`; sleep 100 ms while not connected.
- **ServiceThread** (plugin.cpp:133), 100 ms loop. Every 4 s (`MILLIS_TO_EXPIRE`): if in game and allowed
  (`noAutomoveSpectator` and not spectating) `moveToSeriousChannel()`, then `serverDataDirectory::verify()`.
  Every 4 s `updateUserStatusInfo(true)`. Sleeps 500 ms while TS is not connected.
- **CommandProcessor worker** (CommandProcessor.cpp:207). Created lazily on the first `queueCommand` (:57).
  Waits on a condvar over `std::queue<string>`.
- **Detached threads:** update check (task_force_radio.cpp:360); Piwik per `TRACK` (:228);
  `process_tangent_off` per tangent release or `RELEASE_ALL_TANGENTS` (CommandProcessor.cpp:365, :386);
  MessageBox for a too-short nickname (Teamspeak.cpp:262).
- **One thread per positional playback** (`playbackWavProcessing`, PlaybackHandler.cpp:462).
- **TS client threads:** voice callbacks run on TS audio threads; events on the TS main thread.

## 2. Arma → plugin messages

### 2.1 Transport (original)
- Shared memory `Local\TFARSHAMEM`, created by the plugin (SharedMemoryHandler.cpp:271, :304). Events
  `Local\TFARSHAMEM_EVTREQ` and `Local\TFARSHAMEM_EVTRESP` (manual-reset). Mutex `Local\TFARSHAMEM_MTX`.
- **Layout** (SharedMemoryHandler.hpp:14-67; the comment at :7 is stale): `SharedMemString` =
  `{u32 length; char data[2044]}` = 2048 bytes. Header at 0: `u32 size`, `volatile u16
  nextFreeAsyncMessage` @4, `lastGameTick` @8, `lastPluginTick` @16, `bool configNeedsRefresh` @24.
  Sync request @128, sync answer @2176, async array @4224 (300 entries). Total 618,624 bytes.
- **Game side** (`extensions/task_force_radio_pipe/SharedMemoryTransfer.cpp:250-281`):
  - Messages ending in `~` are appended to the async array with the `~` kept. The extension replies itself:
    `DFRAME~` → `"NEEDCFG"` if `configNeedsRefresh`, else `"OK"`; `MISSIONEND~` → no reply, `shutdown()`
    sets `lastGameTick=0` and empties the queue; any other async → `"OK"`.
  - Sync messages: set the sync request, SignalObjectAndWait (1000 ms), read the answer.
  - Not ready or not connected → `"Not connected to TeamSpeak"`.
  - Plugin counts as connected if `lastPluginTick` is under 1000 ms old.
- **Plugin side:** `getData` prefers the sync request, else pops **one async message LIFO**
  (`position = nextFreeAsyncMessage-- - 1`, SharedMemoryHandler.cpp:14). `waitForRequest` waits on EVTREQ
  (20 ms). Every read sets `lastPluginTick`.
- **Connection detection** (`isConnected`, :227): connected = now − `lastGameTick` < `pluginTimeout` s
  (default 4). `onConnected`/`onDisconnected` debounced to ≤1 per 500 ms. `MISSIONEND` disconnects through
  `lastGameTick=0` (CommandProcessor.cpp:415).

### 2.2 Commands
- Field separator TAB. `toGameCommand` (CommandProcessor.cpp:165) uses an FNV-1a hash switch; names under
  3 chars are `unknown`.
- Sync (`processCommand`, :72) splits only 2 tokens (:78). Unknown → `"UNKNOWN COMMAND"`. A known async
  command sent sync → `"ASYNC COMMAND SENT IN SYNC CONTEXT"`.
- Async (`processAsynchronousCommand`, :224) splits fully, ignores unknown names, returns early without a
  `clientDataDir` for the current server.

| Cmd | Mode | Fields (after the name) | Reply | Side effects / ref |
|---|---|---|---|---|
| `TS_INFO` | sync | `SERVER\|SERVERUID\|CHANNEL\|CHANNELID\|PING\|VERSION` | server name, server UID, my channel name, my channel ID, `PONG`, `PLUGIN_VERSION`; else `FAIL` (:615); errors `"ERROR_GETTING_SERVER_NAME"` | — |
| `POS` | sync + queued async (:87) | nick, `[x,y,z]`, viewdir `[x,y,z]`, canSpeak, canUseSW, canUseLR, canUseDD, vehicleID, terrainInterception, voiceVolume (gain mult), objectInterception, isSpectating, isEnemyToPlayer (14 tokens, unchecked, :263) | 2 chars: c1=`1` if talking now or tangent≠NONE; c2=`1` if `receivingTransmission>0`; `"00"` unknown client (:89-101) | Async: `updatePosition`. Me: `systemset3DListenerAttributes((0,0,0))`. Others: `setGameClientMuteStatus` + `channelset3DAttributes(cid,(0,0,0))` (:593) |
| `IS_SPEAKING` | sync | nick | same 2 chars | — |
| `IS_SPEAKING_BULK` | sync | nick₁…nickₙ | `"XX\t"` per nick, trailing tab kept (:103) | — |
| `RECV_FREQS` | — | — | not mapped → `UNKNOWN COMMAND` (:145 dead) | — |
| `FREQ` | async | swFreqs, lrFreqs, alive, speakVolume (int → myVoiceVolume), nickname, waves, terrainCoef, globalVolume, recvDistMult, speakerDistance (task_force_radio.hpp:47) | — | Sets `gameData`. Renames TS nickname to the game nickname unless `"Error: No unit"`, `"Error: No vehicle"`, `"any"` or equal; fires `onTeamspeakClientUpdated` (:247-259). Frequency list `["<freq><code>\|vol\|stereo\|classname",...]`: strip `[]`, split `,`, strip quotes, split `\|`; 3 or 4 parts (helpers.cpp:307) |
| `KILLED` | async | nick | — | `processUnitKilled` (:531): that client's dataFrame=INVALID; mine INVALID → alive=false; serious-mode mute matrix over my channel (:552-587) |
| `DFRAME` | async | — | NEEDCFG/OK (extension) | `currentDataFrame++` |
| `SPEAKERS` | async | one field: speakers separated by `0x0B`, parts by `0x0A`: radio_id, freqs (`\|`), nick, pos (`[]` = on owner), volume, vehicleID, [waveZ] | — | Rebuilds the `speakers` multimap by frequency (:493). <6 parts aborts the rest. waveZ defaults to pos.z, or 1 if pos null |
| `TANGENT` / `TANGENT_LR` / `TANGENT_DD` | async | PRESSED/RELEASED, freq+code, range, subtype, [classname] | — | §8 PTT. Broadcasts `cmd\t<sw 1/0>\t<dd 1/0>\t<myNick>` (:326) |
| `RELEASE_ALL_TANGENTS` | async | nick | — | Detached `process_tangent_off` with 0 delay; broadcasts `RELEASE_ALL_TANGENTS\t<nick>` (:377) |
| `SETCFG` | async | key, value, type (`BOOL`/`SCALAR`/other) | — | Invalid key → MessageBox `"Used invalid config key: …"`. `BOOL` = `true`/`TRUE`; `SCALAR` = strtof; else string; token count ≠ 4 → `""` (:388) |
| `MISSIONEND` | async | — | — | No-op in the plugin |
| `RadioTwrAdd` | async | `0x0A`-separated `netid;range;[pos]` | — | `addAntenna` (replaces existing ID) (:418) |
| `RadioTwrDel` | async | `0x0A`-separated netIDs | — | `removeAntenna` (:432) |
| `TRACK` | async | action, uid, customVars `[[1,"n","v"],...]` | — | HTTP GET to Piwik, detached (task_force_radio.cpp:212) |
| `collectDebugInfo` | async | — | — | Dumps logs to `%APPDATA%\TS3Client\logs\<dd-mm_HH-MM-SS>\` (:440) |

`NEEDCFG`: `needRefresh` starts true (settings.hpp:215), false on any `set` (:152), true again on game
disconnect (task_force_radio.cpp:24). The addon then resends every `SETCFG`
(`addons/core/functions/plugin/fnc_pluginNextDataFrame.sqf`).

## 3. Audio pipeline

### 3.1 Callbacks
- `onEditPlaybackVoiceDataEvent`: not implemented.
- `onCustom3dRolloffCalculationClientEvent` (plugin.cpp:821): `*volume=1`. TS 3D is neutralised:
  `systemset3DSettings(1,1)` on connect (Teamspeak.cpp:300), clients and listener at (0,0,0) on each POS.
- `onEditPostProcessVoiceDataEvent` (plugin.cpp:747): in game only. Channels≠2 → copy first 2 channels to
  stereo, process, write back, zero the rest. Calls `processVoiceData(sender)`.
- `onEditCapturedVoiceDataEvent` (plugin.cpp:789): only if in game, `*edited & 2`, alive, `speakers`
  non-empty, `myClientData` exists. Copies the mic into stereo, runs `processVoiceData(myId,
  isFromMicrophone=true)`, `appendPlayback("my_radio_voice", raw)`. You hear your own voice from nearby
  speaker radios; outgoing audio unchanged.
- `onEditMixedPlaybackVoiceDataEvent` (plugin.cpp:448 → PlaybackHandler.cpp:36): current server only;
  `!(fillMask & 3)` → zero; mix each playback additively, clamped, into channels 0–1; drop finished ones;
  `fillMask |= 3` if anything written.

### 3.2 `processVoiceData` (plugin.cpp:455-743), per sender
Set FTZ/DAZ. Exit if not in game, no `clientDataDir`, or sender unknown.
1. `hasPluginEnabled = isPluginEnabledForUser` (:421): sender metadata `<TFAR>` must start with
   `"Connected to Game: [B]Yes[/B]\nPlaying: [B]Yes[/B]\n"`. 10 s cache, timestamp refreshed every call
   (:441). Also true on any received plugin command.
2. No `myClientData` or plugin disabled for sender: sender is me → zero; serious mode and alive and in game
   and enabled → zero; else **return untouched** (:480-499).
3. `muteSpectators` and both spectating → zero.
4. Spectators (:512-515): `notHearable = me.spectating && ((enemy && spectatorNotHearEnemies) ||
   (!enemy && !spectatorCanHearFriendlies))`; `pure = both spectating`; if `!(pure || !notHearable)` and
   serious and (I'm dead or sender not alive) → zero.
5. `applyGain(sender.voiceVolumeMultiplifier)`; `originalBuffer = copy`.
6. `shouldPlayerHear = sender.canSpeak && me.canSpeak`; `vehicleVolumeLoss = clamp(myIso + hisIso, 0,
   .99)`; same vehicle = names equal and ≠ `"no"`; `distance = |me-him| + 2·objectInterception`.
7. **Direct speech** if not me, not spectator, not notHearable, `distance <= voiceVolume+15`:
   - shouldPlayerHear and (loss<0.01 or same vehicle): objectInterception>0 and not same vehicle →
     Butterworth LP2 cutoff `2000 - min(u8(n),5)·objectInterceptionStrength`, cached per count, cleared on
     strength change (clientData.cpp:53); run over last frame's `OISampleBuffer`, save current frame, filter
     current at gain 1. Otherwise only save into `OISampleBuffer`. No distance gain here (X3DAudio does it).
   - else shouldPlayerHear (different vehicles): `processFilterStereo(gain = volumeAttenuation(d, true,
     voiceVolume, 1-loss)·(1-loss)^1.2, LP2 cutoff 20000·(1-loss)/4, key "local_vehicle"+loss)`.
   - else `gain = volumeAttenuation(d, false, …)·14`, LP4 100 Hz, key `"local_cantspeak"`.
   - `clunk["voice_clunk"].process(buf, myPos.directionTo(his), myView.toAngle())` (ITD).
   - X3DAudio `applyILD` (helpers.cpp:30).
   - `processCompressor` (SimpleComp: thresh 80 dB raw i16 scale, ratio 0.1, attack 1 ms, release 300 ms,
     48k; clientData.hpp:76).
   - else if not both spectating → zero.
8. **Radios:** `processCompressor(originalBuffer)` (same instance, second run). `listed = spectator ? {} :
   isOverRadio(me,false,false)`; `radioDist = me.effectiveDistanceTo(him)`. Per entry (skip intercom from
   mic): copy original; `volumeLevel = ((vol+1)/10)^4` (helpers.cpp:302).
   - **SW/LR:** headsetLowered ×0.1. Switch on sender subtype (by length):
     - `digital`: over SW `err = min(antLoss, radioDist/range)` → `PersonalRadioEffect[radio_id]`; else
       (underwater/DD) `err = min((uw·range/ddDist + (normal-uw))/range, antLoss)` →
       `UnderWaterRadioEffect`; gain `volumeLevel·0.35`, entry stereo mode.
     - `airborne` → `AirborneRadioEffect`; `digital_lr` → `LongRangeRadioEffect`.
     - `phone` → `processFilterStereo(volumeLevel·10, Butterworth BP2 1850/1550)`.
     - else, incl. `dd` → zero. Null `antLoss` is 7.0.
   - **Ground (speaker):** loss = same vehicle ? 0 : clamp(myIso+spkIso); Butterworth BP1 2000/1000 gain 4;
     `spkDist = vol<20 ? vol/10·speakerDistance : vol·1.9`; attenuation (plain or vehicle LP); `waveZ <
     -1.1` → LP4 100 Hz gain 14; `PersonalRadioEffect[radio_id]` at `volumeLevel·0.35`; `clunk[radio_id]`
     ITD, simple ILD.
   - **Intercom:** `LongRangeRadioEffect["intercom"]` err 0; gain `intercomVolume`, ×0.1 headset lowered,
     ×(1-`intercomDucking`) if any LR entry vol>2; stereo.
   - `mixIntoAdditive` (clamped).
9. Output `applyGain(globalVolume)`.

### 3.3 Formulas
- `volumeAttenuation(d, hear, max, mult=1)` (helpers.hpp:122): d≤1 → 1; `maxD = hear ? max·mult : 5`;
  `g = 10^((d/(2·maxD))·-60/20)`; g<0.001 → 0; else min(1,g).
- Simple ILD (helpers.cpp:12): `dir = atan2(y,x) + viewAngle` (wraps only above 2π); `L =
  -0.37525·cos(dir)+0.625`, `R = 0.37525·cos(dir)+0.625`.
- X3DAudio ILD (helpers.cpp:30-167): `X3DAudioInitialize(FRONT_LEFT|FRONT_RIGHT, speed of sound)`.
  Positions swapped deliberately: listener at sender position, front = my view, top = (front×(0,0,1))×front;
  emitter at my position, front = sender view normalised with x,y negated; 1 channel;
  `CurveDistanceScaler = sender voiceVolume`. Cone (if `voiceCone`): inner 90°, outer 135°, inner volume
  1.8, outer 1.0. 25-point curve: (0,1), (0.05k, atten(vv/20·k)) k=1..19, (1.05, atten(vv)), (1.10,
  atten(1.2vv)), (1.15, atten(1.4vv)), (1.20, atten(1.6vv)), (1.25, 0). `CALCULATE_MATRIX` 1→2;
  `applyStereoGain(m[0], m[1])`.
- `effectiveDistanceTo` (clientData.cpp:125): `(d + T·c + T·c·d/2000)·receivingDistanceMultiplicator`.
- `distanceForDiverRadio` = `70 + 230·(1-wavesLevel)` (helpers.cpp:352).
- `distanceUnderwater` (datatypes.cpp:150): length of the segment below z=0.

### 3.4 Clunk (Clunk.h:10, include/clunk/hrtf.cpp)
Rotates the relative position by θ = `atan2(view.x, view.y)`. Buffers input; above 2048 shorts runs
`Hrtf::process` (window 512, MDCT, Vorbis window, overlap-add), zeros until enough output (latency).
**KEMAR multiply commented out** (`TODO: fix HRTF`). Net effect: both outputs from the **left input
channel**, far ear delayed by `idt = 0.093·(a+sin a)/344·48000` samples.

### 3.5 Radio effects (RadioEffect.hpp)
- `processRadioEffect` (:230): not stereo → gain ×1.5; mono = sum·gain/(channels·32766); `effect.process`;
  out `clamp(-1,1)·32766` to all channels or left/right only.
- `SimpleRadioEffect::process` (:93): `avg = mean|x|`; foldback threshold `0.3·(1-err)·avg/0.005`;
  foldback(x,t) = `||fmod(x−t,4t)|−2t|−t`, t<1e-5 → 0; per sample `x *= 30`, delay line 2400 (effective
  2399, ~50 ms), ring mod `in·(1-err) + in·sin(phase·π/2)·err` (phase += 90/48000, reset >1); RBJ HP then LP.
- `calcErrorLevel` (:137): table `{0,.15,.3,.6,.9,.95,.96,.97,.98,.995,.9978,.9988,.99999}`, `part =
  clamp(int(e·10),0,11)`, `r = t[p] + (t[p+1]-t[p])·(e - p/10)` (missing ×10 on the slope).

  | Effect | High-pass | Low-pass |
  |---|---|---|
  | LR | 520 Hz, Q 0.97 | 1300 Hz, Q 1.0 |
  | Airborne | 1000 Hz, Q 1 | 4000 Hz, Q 1 |
  | Personal (SW) | 900 Hz, Q 0.85 | 3000 Hz, Q 2.0 |

- UnderWater (:56): zero sample if `rand() < RAND_MAX·err` (MSVC 32767), Butterworth BP2 1000/400, ×30.
- `processFilterStereo` (helpers.hpp:152): i16/32767 → float per channel, DSPFilters DirectFormII, out
  `·gain·32766` clipped.

### 3.6 PlaybackHandler
- Path = `pluginPath + name + ".wav"` (PlaybackHandler.cpp:13); only 2-ch 48 kHz S16 accepted; cached (:268).
- Sounds (PlaybackHandler.hpp:290): `radio-sounds/{lr,sw,ab}/{local,remote}_{start,end}` by subtype
  `digital_lr`/`digital`/`airborne` (dd files unused); `radio-sounds/on`/`off` via TS `playWaveFile`.
- Positional variant (:147) → `playbackWavProcessing` on its own thread: gain; vehicle LP if loss>0.01 and
  not same vehicle; on ground: attenuation with `round(radioVol/10·speakerDistance)`, speaker BP gain 4,
  underwater LP4 gain 700; non-null pos: simple ILD only; not on ground: stereo-mode muting. Names
  `fileName + rand()`.

### 3.7 SampleBuffer gain
`applyGain`: 0 → zero, 1 → no-op, else truncating multiply; the SSE path wraps (no saturation).
`mixIntoAdditive` clamps.

### 3.8 Reception routing (`isOverRadio`, clientData.cpp:267)
- Intercom first: `intercomEnabled`, sender tangent NONE, same vehicle ≠ `"no"`, same `intercomSlot` ≠ -1.
- `range==0` → stop. Antenna = `findConnection(senderPos, range, myPos)` when sender is not me.
- Local radio (`isOverLocalRadio`, :148): SW tangent (or ignore flag) and canSW||canDD → underwater (z<0 and
  !canUseSWRadio) ? DD : SW; else LR tangent and canLR → LR. Frequency must be in my LR or SW map.
  Underwater distance remap; antenna fallback. Prefer LR if `me.canUseLRRadio`. Half duplex: blocked if
  that radio's classname equals `currentTransmittingRadio`.
- Speakers: effective distance > range and no antenna → stop. Requires (canSW and SW tangent) or (canLR and
  LR tangent). Speakers on the exact frequency, not owned by the sender, known position, z≥0.
- `AntennaManager::findConnection` (antennaManager.cpp:24): candidates dist²(sender) < range² and dist²(me)
  < antennaRange²; `loss = dMe/antRange + dSender/range`; minimum below 1.0; null connection loss==7.

## 4. TS3 API usage
**Functions used:** `getCurrentServerConnectionHandlerID`, `getServerConnectionHandlerList`,
`getConnectionStatus`, `getClientID`, `getClientVariableAsString` (NICKNAME, META_DATA),
`getClientVariableAsInt`, `getClientSelfVariableAsInt`, `setClientSelfVariableAsString` (NICKNAME,
META_DATA), `setClientSelfVariableAsInt` (`CLIENT_INPUT_DEACTIVATED`), `flushClientSelfUpdates`,
`getChannelOfClient`, `getChannelClientList`, `getChannelList`, `getChannelVariableAsString`
(CHANNEL_NAME), `getServerVariableAsString` (VIRTUALSERVER_NAME, VIRTUALSERVER_UNIQUE_IDENTIFIER),
`requestClientMove`, `requestMuteClients`, `requestUnmuteClients`, `sendPluginCommand`, `playWaveFile`,
`systemset3DSettings`, `systemset3DListenerAttributes`, `channelset3DAttributes`,
`getPreProcessorConfigValue`/`setPreProcessorConfigValue` (`"vad"`), `getPluginPath`, `getConfigPath`,
`getErrorMessage`, `logMessage`, `printMessageToCurrentTab`, `freeMemory`.

**Metadata** (Teamspeak.cpp:713-956): name "Task Force Arrowhead Radio"; version `PLUGIN_VERSION`
("1.0.0.1", version.h); author "[TF]Nkey and Dedmen"; description "Radio Addon for Arma 3". apiVersion is
derived from the TS exe version (≥3.6 → 26). Autoload 1; no configure dialog; command keyword `tfar`. Info
title "Task Force Radio Status"; `infoData` (PLUGIN_CLIENT only) = text between `<TFAR>` and `</TFAR>` in
the client's metadata (512-byte buffer). `onServerErrorEvent` returns 1 with a returnCode. No menus or
hotkeys.

**`/tfar` commands** (Teamspeak.cpp:838): `diag`, `pos`, `full…`, `debug` (only with `allowDebugging`);
`rstflt` always (resets every client's radio effects).

**Own metadata** every 4 s: `<TFAR>Connected to Game: [B]Yes|No[/B]\nPlaying: [B]Yes|No[/B]\nPlugin
version: [B]v[/B]\nAddon version: [B]x[/B]\n</TFAR>`, text outside the tags preserved (Teamspeak.cpp:644).
On disable/shutdown: `[B]Task Force Radio Plugin Disabled[/B]`, then cleared.

**Client-to-client protocol** (`sendPluginCommand`, TAB-separated; receiver checks plugin name starts with
`TFAR` and the server is current, plugin.cpp:1074):

| Message | Target | Receive handling |
|---|---|---|
| `TANGENT[_LR\|_DD]\tPRESSED\|RELEASED\tfreq\trange\tsubtype[\tclassname]\tsw\tdd\tnick` | current channel | 8 or 9 tokens; nick last; `canUseSW = tokens[6]`, `canUseDD = tokens[7]` (shifted by one with 8 tokens — bug); `processTangentPress` (plugin.cpp:879) |
| `RELEASE_ALL_TANGENTS\tnick` | current channel | that client's tangent = NONE |
| `VOLUME\tnick\tvoiceVolumeMeters\ttrue\|false` | current channel, on my `CLIENT_FLAG_TALKING` change (plugin.cpp:829) | sets `voiceVolume`, `pluginEnabled`, `clientTalkingNow`; not me → `setGameClientMuteStatus` |
| `REQVOL\t<clientID>` | `CLIENT` | replies `VOLUME`; never triggered (dead) |

**`processTangentPress`** (plugin.cpp:879): marks sender plugin-enabled, sets subtype, canUseLR (if LR),
canUseSW, canUseDD. `playPressed = pressed` only when the state flips (a repeated PRESSED plays the *end*
sound). Sets tangent type, frequency (`prev_<f>` on release), range (0 on release). For RELEASED the routing
list is computed before the reset with ignore flags `(canSW, canLR)`. Per listed entry (not NONE/intercom,
I'm alive): first press → `receivingTransmission++`, insert frequency, play remote sound at gain
`volMult(vol)·globalVolume` at the entry position; not `playPressed` → remove and decrement. Finally
`setGameClientMuteStatus(…, {true, !empty})`.

## 5. Settings and data models
**Settings** (settings.hpp:11): case-insensitive *prefix* key match (`_strnicmp`, :47); value type fixed by
the default; cross-type set converts (bool←float means >0).

| Key | Default | | Key | Default |
|---|---|---|---|---|
| full_duplex | true | | tangentReleaseDelay | 0 (ms) |
| addon_version | "unknown" | | moveWhileTabbedOut | false |
| serious_channelName | "" | | intercomDucking | 0.2 |
| serious_channelPassword | "" | | minimumPluginVersion | 0 |
| intercomVolume | 0.1 | | objectInterceptionStrength | 400 |
| intercomEnabled | true | | voiceCone | true |
| pluginTimeout | 4 (s) | | allowDebugging | true |
| headsetLowered | false | | noAutomoveSpectator | false |
| spectatorNotHearEnemies | false | | disableAutomaticMute | false |
| spectatorCanHearFriendlies | true | | muteSpectators | false |

The addon sends its values (`addons/core/functions/plugin/fnc_sendPluginConfig.sqf:21-41`), e.g. channel
"TaskForceRadio", password "123", `minimumPluginVersion` 339.

**`gameData`** (task_force_radio.hpp:44): SW/LR maps `freq → {volume, stereoMode, radioClassname}`;
`speakers` multimap `freq → {radio_id, weak client, pos, volume, vehicle, waveZ}`; `myVoiceVolume` 0,
`alive` false, `wavesLevel` 0, `terrainIntersectionCoefficient` 7, `globalVolume` 1,
`receivingDistanceMultiplicator` 1, `speakerDistance` 20; `currentDataFrame` = INT_MAX (INVALID),
`currentTransmittingRadio`, `tangentPressed`, `pttDelay`.

**`clientData`** (clientData.hpp:249): pluginEnabled + check time, clientId, tangent type, voiceVolume,
range, canSpeak, clientTalkingNow, dataFrame, canUse SW/LR/DD, terrain and object interception,
voiceVolumeMultiplifier, spectating, enemy, `receivingTransmission` (u8), `receivingFrequencies`, effects,
`OISampleBuffer`, nickname, position, view direction, last update time, transmitting frequency and subtype,
`vehicleDescriptor {name, isolation, intercomSlot}` + velocity. vehicleID is
`name\x10isolation|"turnout"\x10slot[\x10velocity]` or `"no"` (:349). `getClientPosition` extrapolates by
velocity·elapsed (clientData.cpp:104). `isAlive` (:136): dataFrame within ±1 and update < 20 s; on failure
it sets dataFrame INVALID (side effect).

## 6. sqlite3
At init: `<TS config path>settings.db`, `SELECT value FROM Profiles WHERE
key='Capture/Default/PreProcessing'` (plugin.cpp:344-353). Line by line: `delay_ptt=true` →
`gameData.pttDelay = delay_ptt_msecs` (:259). Only use.

## 7. Debug and optional code
- DebugUI, TextRenderer, Shader, glfw/glad/freetype/glm (`/tfar debug`).
- Tracy (`ProfileFunction`/`ProfileScopeN`/`ProfilerLog`, ProfilerTracy.hpp:24); API profiler.
- Logger: `teamspeakClientlog` only registered with `ENABLE_API_PROFILER` (plugin.cpp:318-328); file logs
  `%APPDATA%\TS3Client\TFAR_{pluginCommands,gameCommands}.log` only on CI builds.
- Diagnostics: `doDiagReport`, `circularLog`, `verboseDataLog`, `collectDebugInfo`, `/tfar diag|pos|full`.
  `messages` grows without bound (clientData.cpp:380).
- Piwik `TRACK` and update check (HTTP
  `raw.github.com/michail-nikolaev/task-force-arma-3-radio/master/current_version.txt`, `Version >` compare,
  MessageBox).

## 8. Subtle behaviour
- **`Vector3D::normalized()` multiplies by `fast_invsqrt(length())`** instead of `invsqrt(lengthSqr)`
  (datatypes.cpp:105): `directionTo` returns length ≈ √distance. Feeds clunk, the simple ILD and the X3DAudio
  emitter front.
- **Subtype matched by string length** (plugin.cpp:116): 2 = `dd`, 7 = `digital`, 8 = `airborne`, 10 =
  `digital_lr`, 5 = `phone`.
- **PTT and VAD** (CommandProcessor.cpp:293-376, :631): acts only on a change of `tangentPressed`.
  `TANGENT_DD` treated as SW. Local sound volume from the SW map for digital, else LR. `!full_duplex` and 6
  tokens → set `currentTransmittingRadio`. Press (recursive `tangentCriticalSection`): if
  `!waitingForTangentOff`: `vadEnabled = check`, disable VAD, `setVoiceDisabled(false)`, broadcast; else
  `skipTangentOff = true`. Release: detached thread sets `waitingForTangentOff`, sleeps `pttDelay +
  tangentReleaseDelay`, then under the lock: not skipping → re-enable VAD if it was on,
  `setVoiceDisabled(!vadEnabled)`, broadcast, waiting=false; else skip=false. `onGameEnd` restores PTT/VAD
  (task_force_radio.cpp:54).
- **Lifecycle:** `onGameConnected`: if not in game, play `on`, in-game, `onGameStart` →
  `moveToSeriousChannel`, `dataFrame = 0`. `onGameDisconnected`: `unmuteAll`; if in game: play `off`,
  `onGameEnd` (move back, reset nickname, unmute, PTT/VAD restore), `setRefresh`; in-game=false;
  alive=false, frame=INVALID (plugin.cpp:339).
- **Serious mode:** active when I'm in the `serious_channelName` channel (`ID:<n>` allowed; else exact name
  via cache, Teamspeak.cpp:518). Cache rebuilt on connect and channel events. `checkIfSeriousModeEnabled`
  runs on every `SETCFG` and channel switch; change → recompute all mutes.
- **Auto-mute** (`setGameClientMuteStatus`, plugin.cpp:68): serious mode and `!disableAutomaticMute`. Mute if
  out of voice range (`voiceVolume+15`) and not reachable by radio, or alive mismatch. Never self. Mute state
  tracked locally; `unmuteAll` unmutes the tracked set; leaving clients are unmuted.
- **Channel moving** (`moveToSeriousChannel`, Teamspeak.cpp:207): skipped if the foreground window class
  lacks "arma" (unless `moveWhileTabbedOut`), channel not found, or already there. Saves the original
  channel, moves with the password. Triggered on game start, every 4 s in game, on reconnect, on
  `serious_channelName` set (10 ms debounce, task_force_radio.cpp:79). `moveFromSeriousChannel` returns with
  an empty password.
- **Nicknames:** leading/trailing spaces → `_` (CommandProcessor.cpp:654); TS nickname forced to the game
  name on each FREQ; minimum 3 chars (one-time MessageBox); restored on game end; client index keyed by an
  FNV-1a hash (collisions return the first match).
- **Version checks:** `minimumPluginVersion` > build number (else 5000) → one-time warning
  (task_force_radio.cpp:86). Init aborts if old `task_force_radio_win32/64` or `TFAR_dev_win64` modules are
  loaded (plugin.cpp:301).
- **Locking:** settings, PlaybackHandler, TeamspeakServerData and the tangent lock are recursive; others SRW.
  `gameData` unlocked before `setCurrentTransmittingRadio` (CommandProcessor.cpp:370). `Teamspeak::serverData`
  is unguarded and effect pointers escape their lock.
- **Missing validation:** POS and FREQ index tokens without checks; `NetID` uses `strchr` on a
  non-terminated `string_view` (datatypes.hpp:190).
