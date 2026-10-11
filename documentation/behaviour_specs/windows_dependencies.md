# Windows dependencies of the original plugins

This lists every Windows-only dependency in the C++ TS plugins and what replaces it in the Rust port. `A/` is
`reference/tfar_0_9_12/ts/src/`, `B/` is `reference/tfar_1_0/ts/src/`.

## 1. The architectural blocker
Both versions bridge game and TS through Win32 named objects (0.9.12: named pipe; 1.0: shared memory,
events, mutex). Under Proton these live inside wineserver and a native Linux `.so` cannot see them. One
side must change: the Rust extensions talk localhost TCP instead (D2, game link protocol).

## 2. Catalogue

| Dependency | Where | Replacement |
|---|---|---|
| Named pipe client (`CreateFile`, `PeekNamedPipe`, `ReadFile`, `WriteFile`) | A/plugin.cpp:507-526, :2022-2112; B/pipe_handler.cpp (unused) | `tfar_game_link::server` |
| Shared memory + events + mutex (`CreateFileMapping`, `MapViewOfFile`, `CreateEvent`, `CreateMutex`) | B/SharedMemoryHandler.cpp:85-304 | `tfar_game_link::server` |
| `std::chrono` time_point inside shared memory (MSVC 100 ns vs libstdc++ ns ABI hazard) | B/SharedMemoryHandler.hpp | gone (no shared memory) |
| `CreateThread`, `Sleep`, `GetExitCodeThread` | A:151, :1868, :2244-2282; B/plugin.cpp:137, :162 | `std::thread`, `std::thread::sleep`, joined at shutdown |
| `CRITICAL_SECTION` (recursive), `SRWLOCK` | A:460-462 (~100 uses); B/Locks.hpp:46-163 | `parking_lot` locks, state designed with no re-entry |
| `InterlockedExchange` | A:1998, :2058 | atomics |
| `GetTickCount` | A (many) | `std::time::Instant` |
| `MessageBox` | A:1972; B/plugin.cpp:307, CommandProcessor.cpp:399, PlaybackHandler.cpp:93-312, SharedMemoryHandler.cpp:49, task_force_radio.cpp:96, :363, Teamspeak.cpp:264 | `printMessageToCurrentTab` + log |
| WinInet (`InternetOpen`, `HttpOpenRequest`, …) | A:124-190; B/task_force_radio.cpp:180-209, :336-357 | `ureq` in `tfar_web_requests` |
| Version resource (`GetModuleFileName`, `GetFileVersionInfo`, `VerQueryValue`) for API detection | A:2139-2171; B/Teamspeak.cpp:707-766 | constant API 26 |
| MASM trampoline `apicompat.asm` for the plugin-command signature | B/apicompat.asm | API 26 signature only |
| `GetModuleHandleA` duplicate-plugin check | B/plugin.cpp:301-306 | dormant-plugin rule (two Rust plugins coexist) |
| `GetForegroundWindow` / `GetClassNameW` "arma" check before channel move | B/Teamspeak.cpp:208-219 | extension sends `FOCUS` frames |
| `X3DAudioInitialize` / `X3DAudioCalculate`, `x3daudio.lib` | B/helpers.cpp:22-165 | `hrtf` crate + `tfar_audio::spatial_audio` (D20) |
| `IsProcessorFeaturePresent` SSE check | B/SampleBuffer.hpp:7 | not needed |
| `QueryPerformanceCounter`, `__declspec(thread)`, `__forceinline` | B/profilers.* | `profiling` crate |
| `OutputDebugStringA`, `__debugbreak`, `DebugBreak` | B/plugin.cpp:173, :181, Logger.cpp:93; many | `tracing` |
| `getenv("appdata") + "\TS3Client\..."` (NULL crash on Linux) | B/plugin.cpp:326, CommandProcessor.cpp:447, Teamspeak.cpp:864 | TS `getConfigPath()` |
| `C:\Windows\Fonts\arial.ttf`, GLFW win32, `opengl32.lib`, `freetype.lib` | B/TextRenderer.cpp:47, DebugUI.cpp | `eframe`/`egui` (X11, glow) |
| `strcat_s`, `strncpy_s`, `_snprintf_s`, `_strnicmp`, `MAX_PATH`, `_wfopen` | A:2250, :2353; B/plugin.cpp:346, Teamspeak.cpp:940, PlaybackHandler.cpp:276-311, settings.hpp:49 | Rust strings |
| `_set_FMA3_enable`, `_MM_SET_FLUSH_ZERO_MODE` | A:2219, :714 | denormal-safe f64 processing |
| Resource file, `version.h` DLL names | B/Resource.rc | Cargo metadata |

## 3. Library portability notes (for the oracle in `tools/tfar_golden_vectors`)
- clunk: `hrtf.cpp:65` calls `pow10f`, which modern glibc removed; needs a shim define.
- dspfilters: the folder is lowercase `dspfilters` but includes use `"DspFilters/..."` (≈38 places) and
  backslashes; the oracle build provides an include alias.
- simpleSource, sqlite3: portable.
- B: `SampleBuffer.cpp:5`, `:24` specialise members without `template<>` (GCC rejects);
  `helpers.hpp:28-37` FNV constants tied to `_WIN64`.

## 4. Build systems
- A: `task_force_radio_plugin.vcxproj` only (v140, `/MT`, `/fp:fast`, defines `CLUNKAPI=`, `CLUNK_USES_SSE`,
  links `wininet.lib`, `version.lib`). Outputs `task_force_radio_win32/64`.
- B: `task_force_radio_plugin.vcxproj` (12 configurations, v142/v143, C++17), links `wininet`, `version`,
  `x3daudio`, `opengl32`, `freetype`; MASM; outputs `TFAR_win32/64`. Tracy is on in Release. `ts/lib/` holds
  only Windows binaries (`freetype.lib`, `X3DAudio.lib`).
- Neither tree has CMake, a Makefile, or CI for the plugin.
