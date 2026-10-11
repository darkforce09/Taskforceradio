# TFAR 0.9.12 TeamSpeak plugin: behaviour spec

This is the functional spec of the C++ plugin at `reference/tfar_0_9_12/ts/src/`: `plugin.cpp` (P),
`RadioEffect.h` (RE), `Clunk.h` (CL), `plugin.h`. It describes the original behaviour, bugs included. The
Rust port fixes the bugs listed in `/documentation/architecture/bug_fixes.md`.

Behaviours most likely to break a faithful reading:
- **KILLED never runs.** The parser needs more than 2 tokens; Arma sends exactly 2, so it returns `FAIL`
  (P:1775).
- **`inGame` depends on the serious channel name.** It only becomes true when a non-VERSION command arrives
  and `serious_mod_channel_name` is non-empty (P:2062). While false, the captured-voice callback and VOLUME
  broadcasts do nothing.
- **Error-level sign bug.** RE:152 uses `(from - to)`, so the curve runs the wrong way.
- **HRTF filtering never applies.** KEMAR filtering is commented out; only input channel 0 is used.
- **Fall-through to `FAIL`.** RELEASE_ALL_TANGENTS and TRACK do their side effect but have no `return`.
- **Splitting follows `std::getline`.** A trailing empty field is dropped.

## 1. Structure, state, threads, locking

### Constants (P:40-77)
| Constant | Value |
|---|---|
| `RADIO_GAIN_LR` / `RADIO_GAIN_DD` | 5 / 15 (unused) |
| `CANT_SPEAK_GAIN` | 14 |
| `MAX_CHANNELS` | 8 |
| `PLUGIN_API_VERSION` | 21 |
| Pipe | `\\.\pipe\task_force_radio_pipe` |
| `PLUGIN_NAME` | `"task_force_radio"` |
| `MILLIS_TO_EXPIRE` | 4000 |
| `DD_MIN` / `DD_MAX` | 70 / 300 |
| `UNDERWATER_LEVEL` | -1.1 |
| `PLUGIN_VERSION` | `"0.9.12"` |
| `CANT_SPEAK_DISTANCE` | 5 |
| `SPEAKER_GAIN` | 4 |
| `INVALID_DATA_FRAME` | 9999 |

Enums (P:79-93): `OVER_RADIO_TYPE` SW, LR, DD, NONE; `LISTED_ON` SW, LR, DD, NONE, GROUND.

### Helpers
- `versionNumber` (P:95): each digit is added; each `.` multiplies the running total by 10.
- `parseArmaNumber` = `atof`; `parseArmaNumberToInt` = `round(atof)` (P:112-120).
- `split` (P:801, 1318): `std::getline` semantics; trailing empty token dropped; empty string → 0 tokens.
- `startWith(prefix, s)` (P:494): requires `s.size() > prefix.size()` (strict).

### `CLIENT_DATA` (P:192-366), one per nickname
- Fields: `pluginEnabled`, `pluginEnabledCheck` (tick), `clientId`, `tangentOverType`, `clientPosition`
  (relative to the listener's eye), `positionTime`, `frequency`, `voiceVolume` (int m, from VOLUME), `range`,
  `canSpeak`, `clientTalkingNow`, `dataFrame`, `canUseSW/LR/DDRadio`, `subtype`, `vehicleId`,
  `terrainInterception` (int), `viewAngle` (deg), `voiceVolumeMultiplifier`.
- Lazy DSP maps: `swEffects` (PersonalRadioEffect), `lrEffects` (LongRangeRadioffect), `ddEffects`
  (UnderWater), `clunks`; `filtersCantSpeak` Butterworth LowPass<4> `setup(4,48000,100)` (P:297);
  `filtersVehicle` LowPass<2> `setup(2,48000,20000*(1-loss)/4)` keyed `key+to_string(loss)` (P:307);
  `filtersSpeakers` BandPass<1> `setup(1,48000,2000,1000)` (P:243); `filtersPhone` BandPass<2>
  `setup(2,48000,1850,1550)` (P:232). All 8-channel `SimpleFilter`s.
- `compressor`: SimpleComp 48000, threshold 65 dB, release 300 ms, attack 1 ms, ratio 0.1 (P:357-362).
- `resetRadioEffect()` clears sw/lr/dd effects plus speaker and phone filters (P:319). `resetVoices()`
  clears clunks plus cant-speak and vehicle filters (P:333).
- Defaults: `canSpeak = true`, `clientId = -1`, others 0/false, `tangentOverType = NONE`, multiplier 1.0.

### Other structs (P:370-441)
- `FREQ_SETTINGS {volume, stereoMode}`; `SPEAKER_DATA {radio_id, nickname, pos (0 or 3 values), volume,
  vehicle pair, waveZ}`; `LISTED_INFO {over, on, volume, stereoMode, radio_id, pos, waveZ, vehicle}`;
  `SERVER_PLAYBACK` map of interleaved stereo deques.
- `SERVER_RADIO_DATA`: `myNickname`, `myOriginalNickname`, `tangentPressed`, `myPosition`,
  `nicknameToClientData`, `mySwFrequencies`, `myLrFrequencies`, `myDdFrequency`, `speakers` (multimap by
  freq), `waves` (WAV cache), `ddVolumeLevel`, `myVoiceVolume`, `alive`, `canSpeak`, `wavesLevel`,
  `terrainIntersectionCoefficient` = 7, `globalVolume` = 1, `receivingDistanceMultiplicator` = 1,
  `speakerDistance` = 20, `serious_mod_channel_name`, `serious_mod_channel_password`, `addon_version`,
  `currentDataFrame` = 9999.

### Globals (P:443-466)
`pluginPath`, thread handles, `exitThread`, `pipeConnected`, `inGame`, `pttDelay`, `pttDelayMs`,
`notSeriousChannelId` (`uint64(-1)`), `vadEnabled`, `pluginID`, three `CRITICAL_SECTION`s, `serverIdToData`,
`serverIdToPlayback`, scratch `floatsSample[8][1]` (P:45), tangent state `skip_tangent_off` /
`waiting_tangent_off` (P:1703), `ptt_arguments` (P:1687), timers `lastInGame`, `lastCheckForExpire`,
`lastInfoUpdate` (P:1964-1966).

### Locking
`serverDataCriticalSection` is recursive and re-entered heavily (server data, DSP objects, `floatsSample`).
`playbackCriticalSection` guards playback queues; `tangentCriticalSection` the tangent-off handshake. Many
reads happen outside the lock (P:1837, 2682, 3022, erase on disconnect P:2424).

### Threads
- **PipeThread** (P:2022-2112):
  1. Invalid handle → open the pipe as a client (message read mode).
  2. Peek; if bytes, read ≤4096 and `processGameCommand`.
  3. Command not starting with `"VERSION"` → `lastInGame = now`; if `!inGame` and serious channel name
     non-empty: play `radio-sounds/on`, `inGame = true`, `onGameStart`, `currentDataFrame = 0`.
  4. Write the response with `len + 1` bytes (NUL included).
  5. Read fails → `pipeConnected = false`, sleep 1 s, reopen (old handle leaked).
  6. Peek fails → if connected, `pipeConnected = false` and `updateUserStatusInfo(true)`; sleep 1 s, reopen.
  7. After 50 empty polls, sleep 1 ms.
- **ServiceThread** (P:1977-2019), every 100 ms:
  - Every 4 s: compute `isSerious`, `removeExpiredPositions`; if connected: in game → `setMuteForDeadPlayers`;
    always `updateNicknamesList`.
  - No pipe traffic for 4 s: unmute everyone unless my channel is an "other radio" channel; reset
    `lastInGame`; if `inGame`: play `radio-sounds/off`, `alive = false`, `currentDataFrame = 9999`,
    `onGameEnd`; `inGame = false`.
  - Every 4 s: `updateUserStatusInfo(true)`.
- **UpdateThread** (P:1968): one-shot update check, MessageBox if newer.
- **trackPiwikThread** (P:124): one-shot per TRACK.
- **process_tangent_off** (P:1706): one-shot per local tangent release.

### Expiry and data frames
VERSION increments `currentDataFrame` (P:1795); POS stamps `client.dataFrame`. `hasClientData` requires a
matching `clientId` and `|currentDataFrame - dataFrame| <= 1` (P:584). `removeExpiredPositions` deletes
when `now - positionTime > 20000` or the frame difference > 1 (P:1938).

## 2. Arma pipe commands (`processGameCommand`, P:1769-1936)
Tokens split on `\t`. Server ID is always `getCurrentServerConnectionHandlerID()`. Every message gets a reply
(no async).

| Command (tokens) | Fields | Side effects | Response |
|---|---|---|---|
| `TS_INFO` (==2) | `SERVER` / `CHANNEL` / `PING` | `ts_info` (P:1534) | server name or `ERROR_GETTING_SERVER_NAME`; channel name or `""`; `PONG`; else `FAIL` |
| `KILLED` (**>2**) | `[1]` name | dataFrame = 9999, `setMuteForDeadPlayers` (P:1562) | `DONE` (in practice `FAIL`) |
| `SPEAKERS` (>=1) | `[1]` records split by VT (0x0B), each split by LF (0x0A): radio_id, freqs (`\|`), owner nick, `"[x,y,z]"` or `"[]"`, volume, vehicleId, optional waveZ (default 1) | Clears and rebuilds speakers (P:1728); empty records skipped (first always empty) | `OK` |
| `RELEASE_ALL_TANGENTS` (==2) | `[1]` nick | Forwards raw command via `sendPluginCommand` (CURRENT_CHANNEL) | falls through to `FAIL` |
| `VERSION` (==4) | addon_version, serious channel name, password | stores; `currentDataFrame++` | `OK` |
| `TRACK` (==2) | URL path | Piwik HTTP GET | falls through to `FAIL` |
| `POS` (==14) | `[1]` nick, `[2..4]` x y z (`stof`, relative to listener eye, own = 0,0,0), `[5]` viewAngle deg (own unit only), `[6]` canSpeak, `[7]` useSw, `[8]` useLr, `[9]` useDd (`"true"`), `[10]` vehicleId, `[11]` terrainInterception (`stoi`), `[12]` voiceVolume multiplier, `[13]` currentDirection (my facing, deg) | `processUnitPosition` (P:1580) | `SPEAKING` / `NOT_SPEAKING` |
| `TANGENT` / `TANGENT_LR` / `TANGENT_DD` (==5) | `[1]` PRESSED/RELEASED, `[2]` freq+code, `[3]` range, `[4]` subtype (`digital`, `digital_lr`, `dd`, `airborne`, `phone`) | local PTT flow (P:1812-1873) | `OK` |
| `FREQ` (==14) | `[1]` SW list, `[2]` LR list (Arma `str()` arrays `["freqcode\|vol\|stereo",...]`), `[3]` DD freq, `[4]` alive, `[5]` myVoiceVolume, `[6]` ddVolumeLevel (int), `[7]` nickname, `[8]` wavesLevel, `[9]` terrain coefficient, `[10]` globalVolume, `[11]` voiceVolume (**ignored**), `[12]` receivingDistanceMultiplicator, `[13]` speakerDistance | stores; nickname differs, non-empty and not `"Error: No unit"`/`"Error: No vehicle"`/`"any"` → set `CLIENT_NICKNAME`, `myOriginalNickname = old` on **every** rename (P:1894-1906) | `OK` |
| `IS_SPEAKING` (==2) | nick | read-only | `SPEAKING` if TS talking flag, tangent ≠ NONE, or `clientTalkingNow`; else `NOT_SPEAKING` |
| other | | | `FAIL` |

`parseFrequencies` (P:1515): strip outer `[]`, split `,`, strip quotes, split `|`; only exactly 3 parts
kept: `map[freq] = {round(vol), round(stereo)}`.

### POS (`processUnitPosition`)
- Mine: update my `CLIENT_DATA` (position, time, flags, vehicle, terrain, dataFrame, `viewAngle = [5]`,
  multiplier); set server `myPosition` and `canSpeak`; `systemset3DListenerAttributes(zero, NULL, NULL)`.
- Other: unknown nickname → `updateNicknamesList` first; update fields except `viewAngle`; `myData.viewAngle
  = [13]`; `setGameClientMuteStatus`; `channelset3DAttributes(clientId, zero)`.

### Local TANGENT flow
1. Under the server lock: `changed = tangentPressed != pressed`; store; my `canUseXRadio = true`, `subtype`.
2. `changed` → broadcast = `command + "\t" + myNickname` (6 tokens).
3. Pressed: play `radio-sounds/{lr|dd|sw|ab}/local_start` via `playWaveFile` (`digital_lr` → lr, `dd` → dd,
   `digital` → sw, `airborne` → ab). Tangent lock: if `!waiting_tangent_off`: `vadEnabled = hlp_checkVad()`;
   disable VAD if on; `disableVoiceAndSendCommand(true)`. Else `skip_tangent_off = true` (no broadcast).
4. Released: store `ptt_arguments`; play `local_end`; spawn `process_tangent_off`: waiting = true; sleep
   `pttDelayMs` if `pttDelay`; lock; `!skip` → re-enable VAD if it was on, `disableVoiceAndSendCommand(false)`,
   waiting = false; else skip = false (waiting stays true).
5. `disableVoiceAndSendCommand` (P:1689): `CLIENT_INPUT_DEACTIVATED` = `INPUT_ACTIVE` if `pressed ||
   vadEnabled` else `INPUT_DEACTIVATED`; `flushClientSelfUpdates`; `sendPluginCommand(pluginID, broadcast,
   CURRENT_CHANNEL)`.

## 3. Audio pipeline

### Callbacks
- `onCustom3dRolloffCalculationClientEvent` (P:2892): `*volume = 1.0`.
- `onConnectStatusChangeEvent` (P:2398): ESTABLISHED → reset server data and playback, `myNickname`,
  `updateNicknamesList`, `systemset3DSettings(1.0, 1.0)`; DISCONNECTED → erase server data.
- `onEditPostProcessVoiceDataEvent` (P:2831): channels 0–1 into stereo, Stereo function (P:2653), copy back,
  zero channels ≥2. Reads `samples[q*channels+1]` even for mono input.
- `onEditCapturedVoiceDataEvent` (P:2852): `!inGame` → return; needs `*edited & 2`, `hasClientData(me)`,
  `alive`. Mono→stereo, Stereo function with `clientID = myId`, append to playback `"my_radio_voice"`.
  Captured samples unchanged. Only ground speakers on my transmit frequency owned by others produce sound.
- `onEditMixedPlaybackVoiceDataEvent` (P:2600): playback lock; `!(*channelFillMask & 3)` → zero; per frame
  saturating-add queued samples into channels 0–1, set 2..n to 0; drop empty queues; `fillMask |= 3`.
- `onClientSelfVariableUpdateEvent` (P:2897): `CLIENT_FLAG_TALKING` change while `inGame` → broadcast
  `VOLUME\t<myNick>\t<myVoiceVolume>\t<true|false>`.

### Stereo function (P:2653-2828)
1. FTZ/DAZ; create server entry if missing; read `alive`, my `canSpeak`.
2. Gate: `hasClientData(client) && hasClientData(me) && isPluginEnabledForUser(client)`.
   `isPluginEnabledForUser` (P:2526): metadata between `<TFAR>` and `</TFAR>` must start with
   `[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]Y[/B]`; positive cached 10 s.
3. Speaker in the serious channel and I'm `!alive` → silence.
4. Otherwise:
   1. `applyGain(voiceVolumeMultiplifier)`; copy to `original`.
   2. `shouldPlayerHear = data.canSpeak && myCanSpeak`.
   3. Vehicles, `getVehicleDescriptor` (P:967): contains `_turnout` → name = prefix, loss 0; contains `_` →
      name before last `_`, loss = `stof` after; else whole id, loss 0. `loss = clamp(myLoss + hisLoss, 0,
      0.99)`; `vehicleCheck = same name`.
   4. `d` = Euclidean distance `myPosition`–client.
   5. Direct voice if not me and `d <= data.voiceVolume`: `getClunk("voice_clunk").process(pos, dir =
      myData.viewAngle)`, `applyILD`; shouldPlayerHear: loss < 0.01 or same vehicle → `gain(vfd(d, true,
      voiceVolume))`, else LowPass<2> `"local_vehicle"` gain `vfd(d, true, voiceVolume, 1 - loss) · (1 -
      loss)^1.2`; not hearing → LowPass<4> 100 Hz `"local_cantspeak"` gain `vfd(d, false, ...) · 14`. Gate
      fails (me, or too far) → zero.
   6. Radio: `processCompressor(data.compressor, original)` (stereo-linked, raw shorts as doubles);
      `listed = isOverRadio(false,false,false)`; `radioDistance = effectiveDistance`. Per info: `rb =
      original`; `vol = ((v + 1) / 10)^4`; effect by the **transmitter's** subtype:
      - `digital`: SW effect `radio_id`, `setErrorLeveL(effectErrorFromDistance(over, radioDistance))`,
        `processRadioEffect(vol · 0.35, stereoMode)`.
      - `digital_lr` / `airborne`: same with the LR effect.
      - `dd`: `ddVol = volumeMultiplifier(ddVolumeLevel)`, error from raw distance, gain `ddVol · 0.6`.
      - `phone`: BandPass<2> phone filter gain `vol · 10`.
      - else silence.
      - `info.on == GROUND`: `dist = |myPosition - info.pos|`; BandPass<1> gain 4; `speakerDist =
        info.volume / 10 · speakerDistance` (truncated to int); distance gain or vehicle LowPass with
        `clamp(myLoss + info.vehicle.loss)`; `waveZ < -1.1` → LowPass<4> gain 14; clunk `radio_id` at
        `info.pos`, ILD.
      - saturating mix into samples (P:2564).
   7. `globalVolume` gain.
5. Gate failed, not me: client's channel not an "other radio plugin" channel and is the serious channel and
   `alive && inGame && pluginEnabled(client)` → silence; else untouched. Debug log every 4 s.
6. Gate failed, me → silence.

### Formulas
| Function | Formula |
|---|---|
| `volumeFromDistance` (P:698) | d ≤ 1 → 1. `maxD = hear ? clientDist · mult : 5`; `g = d^-0.3 · max(0, maxD - d) / maxD`; g < 0.001 → 0; else min(1, g) |
| `effectiveDistance` (P:954) | `(d + ti·coef + ti·coef·d/2000) · receivingDistanceMultiplicator` |
| `distanceForDiverRadio` (P:932) | `70 + 230 · (1 - wavesLevel)` |
| `effectErrorFromDistance` (P:941) | `d / maxD`; maxD = range for SW/LR, diver distance for DD |
| `applyILD` (P:608) | `dir = atan2(y, x) + view°·π/180`, wrapped if > π; `L = -0.375·cos(dir) + 0.625`, `R = 0.375·cos(dir) + 0.625` |
| `processFilterStereo` (P:650) | per frame `s / 32767` into an 8-channel filter, × gain, clip [-1,1], × 32766 (32767 at ±1) |
| `processRadioEffect` (P:2476) | mono `(short)(sum / ch)`, `/ 32767 · gain`, effect, clip, `· 32766`; stereoMode 1 = left only, 2 = right only (gain × 1.5), others zeroed |

### `isOverLocalRadio` (P:989)
`d = effectiveDistance`, `radio_id = "local_radio"`, `pos = 0`, `vehicle = my vehicle`. Checks in order (a
later match overwrites): SW on my LR list; SW on SW; LR on SW; LR on LR; DD. SW/LR need the matching tangent
(or ignore flag), the frequency in the list, `his.canUseX && my.canUseY`, `d < range`. DD needs
`myDdFrequency == freq` and raw distance below the diver distance; volume `ddVolumeLevel`, stereo 0.

### `isOverRadio` (P:1091)
Local result if not me. Then if `d < range` and (`canUseSW` and SW tangent or flag, or `canUseLR` and LR
tangent or flag): per speaker with `key == freq` and owner ≠ transmitter add a `GROUND` entry; position is the
explicit one or the owner's position if the owner's frame is fresh.

### RadioEffect.h
- Base (RE:10-54): delay line 2400 (writes `pos`, reads `pos + 1` → 2399 effective); per-sample filter.
- UnderWater (RE:56-89): zero sample if `rand() < RAND_MAX · err` (32767); Butterworth BandPass<2>
  `setup(2, 48000, 1000, 400)`; ×30.
- SimpleRadioEffect::process (RE:101): `x = mean(|buf|) / 0.005`; per sample delay; ringmod(err) `in·(1 -
  m) + in·sin(phase·π/2)·m` (phase += 90/48000, reset > 1); foldback(`0.3·(1 - err)·x`, < 1e-5 → 0, `|in| >
  t` → `||fmod(in - t, 4t)| - 2t| - t`); RBJ HP, RBJ LP, ×30.
- `calcErrorLevel` (RE:144): table `{0, .15, .3, .6, .9, .95, .96, .97, .98, .995, .9978, .9988, .999}`;
  `part = int(e·10)`; `from + (from - to)·(e - part/10)` (sign bug).
- Filters: LongRange RBJ HP 520/0.97, LP 1300/1.0. Personal: mic HP 900/0.85 and mic LP 3000/2.0 first, then
  speaker HP 520/0.97 and LP 4300/2.0 (mic filters before the mean). Airborne uses LongRange.

### Clunk (CL)
1. Rotate position by `direction` degrees: `x' = x·cosθ - y·sinθ`, `y' = x·sinθ + y·cosθ`.
2. Append input to `input_buffer`.
3. Above `512·ch·2` (2048 stereo): process with `out = in - 2048`; `Hrtf::process` returns frames consumed;
   tail pushed back; output appended to the deque.
4. Emit `sampleCount·ch`, zero-filling when short (startup latency). One-shot WAVs under 2048 shorts are
   silent.

`clunk::Hrtf::process` (include/clunk/hrtf.cpp:95-245): pass-through at position (0,0,0); KEMAR lookup only
checks data exists, filtering commented out (hrtf.cpp:218), `amp[]` unused. ITD: `angle = π/2 - atan2(y,
x)`; `t = -0.093·(a + sin a)/344`; `idt_offset = int(t·48000)`; right uses the offset if ≤ 0, left if ≥ 0.
Per window (N = 512, hop 256): source channel 0 / 32768, Vorbis window, MDCT (FFT 128), IMDCT, window,
overlap-add, clip, × 32767.

### WAV playback
- `playWavFile(name)` (P:787): `ts3Functions.playWaveFile(pluginPath + name + ".wav")` (local sounds,
  on/off).
- Positional `playWavFile` (P:711): `clunk::WavFile` (PCM); cached only if stereo 48 kHz else skipped;
  apply gain; `id = path + rand()`; with `hasClientData(me)`: `loss > 0.01 && !vehicleCheck` → vehicle
  LowPass `vfd(dist, true, int(speakerDist), 1 - loss) · (1 - loss)^1.2`; on ground: distance gain,
  BandPass<1> gain 4, underwater LowPass<4> gain 700; always one-shot clunk at `pos` then ILD (local radios
  panned by the view angle); queue under `id`; filters leak per id.
- Remote sounds from `processTangentPress` (P:3025-3044): `radio-sounds/{sw,lr,dd,ab}/remote_{start,end}`
  (phone none), gain `volumeMultiplifier(vol) · globalVolume`.

## 4. TeamSpeak API surface
| Item | Value |
|---|---|
| `ts3plugin_name` | `"Task Force Arma 3 Radio"` (P:2128) |
| `ts3plugin_version` | `"0.9.12"` |
| `ts3plugin_author` | `"[TF]Nkey"` |
| `ts3plugin_description` | `"Radio Addon for Arma 3"` |
| `offersConfigure` | NO |
| `commandKeyword` | `""`; `processCommand` returns 0 |
| `requestAutoload` | 1 |
| `infoTitle` | `"Task Force Radio Status (0.9.12)"` |
| `infoData` | PLUGIN_CLIENT only: text between `<TFAR>` and `</TFAR>`, max 512 bytes |
| `onServerErrorEvent` | 1 with a return code |
| `onServerPermissionErrorEvent` | 0 |

API version via the TS exe version (P:2139); `getPluginPath` 2- or 3-argument (P:2221); no menus or hotkeys.

**Functions used:** `channelset3DAttributes`, `flushClientSelfUpdates`, `freeMemory`, `getChannelClientList`,
`getChannelList`, `getChannelOfClient`, `getChannelVariableAsString(CHANNEL_NAME)`, `getClientID`,
`getClientList`, `getClientSelfVariableAsInt(CLIENT_FLAG_TALKING)`, `getClientVariableAsInt`,
`getClientVariableAsString(NICKNAME / META_DATA)`, `getConfigPath`, `getConnectionStatus`,
`getCurrentServerConnectionHandlerID`, `getErrorMessage`, `getPluginPath`, `getPreProcessorConfigValue("vad")`,
`getServerVariableAsString(VIRTUALSERVER_NAME)`, `logMessage(msg, level, "task_force_radio", 141)`,
`playWaveFile`, `requestClientMove`, `requestMuteClients`, `requestUnmuteClients`, `sendPluginCommand`,
`setClientSelfVariableAsInt(CLIENT_INPUT_DEACTIVATED)`, `setClientSelfVariableAsString(NICKNAME / META_DATA)`,
`setPreProcessorConfigValue("vad", "true"/"false")`, `systemset3DListenerAttributes`, `systemset3DSettings`.

### Plugin commands (CURRENT_CHANNEL; receiver checks `strncmp(pluginName, "task_force_radio", 16)` and the
current server, P:3112)
- `TANGENT` / `_LR` / `_DD` (6 tokens): `type, PRESSED/RELEASED, freq, range, subtype, nickname` →
  `processTangentPress` (P:2955): set `positionTime`, `pluginEnabled`, `subtype`, `canUseX`; flip-based
  play flags; update tangent type, frequency, range; with data: for each `isOverRadio(ignoreSw = isSW,
  ignoreLr = isLR, ignoreDd = isDD)` result `setGameClientMuteStatus`, play remote sound if alive; released
  and alive → `resetRadioEffect`.
- `RELEASE_ALL_TANGENTS` (2 tokens): that nickname's tangent = NONE.
- `VOLUME` (4 tokens): `nick, voiceVolume, true/false` → `voiceVolume` (`stoi`), `pluginEnabled`,
  `pluginEnabledCheck`, `clientTalkingNow`; not me with data → `setGameClientMuteStatus`.

### Serious mode and channel moves
- `onGameStart` (P:1418): remember channel; find channel named `serious_mod_channel_name`;
  `requestClientMove(me, channel, password)`; on success `notSeriousChannelId = previous`.
- `onGameEnd` (P:1391): move back if set and elsewhere; reset to -1; `unmuteAll`; clear name and password;
  restore nickname to `myOriginalNickname` (even empty); clear it.
- `isSeriousModeEnabled(client)`: name non-empty and the client is in that channel.
- `setGameClientMuteStatus` (P:1188): outside serious mode always unmutes (`requestUnmuteClients` on every
  POS). Serious: mute if either data entry missing or I'm not alive; else if not heard over any radio, mute if
  `dist > voiceVolume` or not talking. Muting calls `data.resetVoices()` (null deref when data missing,
  P:1214).
- `setMuteForDeadPlayers` (P:1356): clients in my channel without fresh data muted when `alive && serious`.
- `isOtherRadioPluginEnabled` (P:896): channel name in `RT`, `RT_A2`, `RT_OFP`, `RT_A3`, `Radio`, `[ARMA3.RU]
  Radio`, `PvP_WOG`.
- `updateNicknamesList` (P:1478): nickname → clientId for my channel; called from init, connect, channel,
  client-update, move and kick events (P:2428-2458).

### Metadata (`CLIENT_META_DATA`)
- `setMetaData` (P:1264): replace or insert `<TFAR>…</TFAR>` keeping surrounding text, then
  `flushClientSelfUpdates`.
- Content (P:1300): `[I]Connected[/I] [B]Y|N[/B] [I]Play[/I] [B]Y|N[/B]` then `[/B]
  [I]P:[/I][B]0.9.12[/B][I] A: [/I][B]<addon>[/B]` (the doubled `[/B]` is in the source).
- Disabled: `[B]Task Force Radio Plugin Disabled[/B]` at shutdown.

### Init and shutdown
- `ts3plugin_init` (P:2217): plugin path, locks, `updateNicknamesList` if connected, start Pipe, Service and
  Update threads, read PTT delay from sqlite.
- `ts3plugin_shutdown` (P:2264): `onGameEnd` if in game; `exitThread`; sleep 1 s; `alive = false`;
  `updateUserStatusInfo(false)`; `unmuteAll`; free `pluginID`.

## 5. sqlite and network
- sqlite (P:2248-2258): `getConfigPath() + "settings.db"`, `SELECT value FROM Profiles WHERE
  key='Capture/Default/PreProcessing'`; line `delay_ptt=true` → `pttDelay`; `delay_ptt_msecs=<n>` →
  `pttDelayMs`.
- Update check (P:155): `GET http://raw.github.com/michail-nikolaev/task-force-arma-3-radio/master/current_version.txt`
  when online; body < 10 chars → `versionNumber(remote) > versionNumber("0.9.12")` → MessageBox "New version
  of Task Force Arrowhead Radio is available. Check radio.task-force.ru/en".
- Piwik (P:124): HTTP GET to `nkey.piwik.pro`, path from the TRACK argument, response discarded.

## 6. Library code actually used
- clunk: `Hrtf` (hrtf.cpp, mdct_context.h, fft_context.h, sse_fft_context with `CLUNK_USES_SSE`, Vorbis
  window), kemar.c (pointer only), `Buffer`, `v3f`, `WavFile`.
- DSPFilters via `SimpleFilter<…, Channels>` DirectFormII: Butterworth LowPass<4>, LowPass<2>, BandPass<1>,
  BandPass<2>; RBJ HighPass, LowPass (`setup(sr, fc, q)`). Channels 1 (RadioEffect) and 8 (plugin).
- simpleSource: `SimpleComp` only — stereo `process(l, r)` takes the louder rectified channel + `DC_OFFSET =
  1e-25`, dB via `20/ln10·ln`, `over = max(0, key - thresh)`, envelope `coef = exp(-1000/(ms·sr))`, `gr =
  dB2lin(over·(ratio - 1))`.
- sqlite3 amalgamation; `teamlog/logtypes.h` `LogLevel`.

## 7. Differences from the 1.0 tree
- Bundled libraries effectively unchanged (dspfilters `tr1` alias and an array overload; simpleSource `max`
  → `MAX`; clunk casts, `auto_ptr` → `unique_ptr`; sqlite identical).
- `Clunk.h` changed: `vector` not `deque`, radians; 1.0 zeroes the **whole block** when output is short.
- `RadioEffect.hpp` changed: order 0.9 delay → ringmod → foldback → HP → LP → ×30, 1.0 foldback → ×30 →
  delay → ringmod → HP → LP; `calcErrorLevel` sign fixed in 1.0, index clamped, last entry .99999;
  Personal mic filters removed, speaker filters HP 900/0.85, LP 3000/2.0; new Airborne HP 1000/1.0, LP
  4000/1.0; `processRadioEffect` uses 1/32766 and a float mono sum.
- Elsewhere: compressor threshold 80 dB (1.0) vs 65; `volumeAttenuation` 10^-dB curve replaces `d^-0.3`; ILD
  ±21.5° plus X3DAudio; object-interception filters new in 1.0. Phone, speaker, cant-speak, vehicle, DD
  setups and gains unchanged.
