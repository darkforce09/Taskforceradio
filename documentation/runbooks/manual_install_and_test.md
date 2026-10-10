**Status:** live

# Install the Rust plugins and extensions by hand and test them

Builds both mods' artifacts, installs the plugins into the Flatpak TeamSpeak and the extension DLL
into an installed mod, and checks the link from TeamSpeak, from Proton's wine and from Arma. No
script copies anything into TeamSpeak or a mod folder (decision D13); every copy below is done by
the owner. Run the build from the repository root,
`/run/media/system/Main_storage/Projects/Taskforceradio`.

Every step below works with the code of M8: both plugins and both extensions are complete, and the
mock scenarios of both versions pass. What is left is this live check, deferred by the owner.

## Prerequisites

- The toolchain from `rust-toolchain.toml` with the `x86_64-pc-windows-gnu` target, and
  `x86_64-w64-mingw32-gcc` on `PATH`.
- TeamSpeak 3.6.2 as the Flatpak `com.teamspeak.TeamSpeak3`; its plugins folder is
  `~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/`.
- Arma 3 under Proton (Steam app 107410) with the mod to test installed: TFAR 1.0 or TFAR 0.9.12.
  Its mod root is the folder that holds mod.cpp and the addons folder (for a Workshop install, under
  Steam's workshop content folder for 107410). This runbook writes it as `<installed mod root>`;
  `reference/` in this repository is the read-only archive of the originals, never an install
  target.
- Proton 10.0 in Steam's common folder, for the probe in step 7:
  `~/.local/share/Steam/steamapps/common/Proton 10.0/files/bin/wine`.
- A backup folder outside the mod and TeamSpeak folders, for the original files this runbook
  replaces.

## Dist layout

`cargo xtask dist` writes only under the target folder (`target/dist/`, or `$CARGO_TARGET_DIR/dist/`
when that variable is set):

```text
target/dist/
├── tfar_0_9/
│   ├── task_force_radio_linux_amd64.so   the 0.9 TeamSpeak plugin ("Task Force Arma 3 Radio")
│   ├── task_force_radio_pipe_x64.dll     the 0.9 Arma extension
│   ├── radio-sounds/                     18 WAV files: on, off and sw/, lr/, dd/, ab/
│   └── tfar_hrir/                        IRC_1002_C.bin and CREDITS.md
└── tfar_1_0/
    ├── TFAR_linux_amd64.so               the 1.0 TeamSpeak plugin ("Task Force Arrowhead Radio")
    ├── task_force_radio_pipe_x64.dll     the 1.0 Arma extension
    ├── radio-sounds/                     the same 18 files, byte for byte
    └── tfar_hrir/                        the same sphere and credits
```

TeamSpeak takes the plugin name from the file name, and the originals accept plugin commands only
from `task_force_radio…` (0.9) and `TFAR…` (1.0) senders, so the `.so` files keep these names.

## Steps

1. Build every artifact.

   ```bash
   cargo xtask dist
   ls -R target/dist
   ```

   **Expected:** the listing matches the [dist layout](#dist-layout): two version folders, each with
   its `.so`, `task_force_radio_pipe_x64.dll`, `radio-sounds/` (18 files) and `tfar_hrir/` (2 files).

2. Close TeamSpeak, then remove the old Windows plugin files from its plugins folder by hand.

   ```bash
   flatpak kill com.teamspeak.TeamSpeak3
   ls ~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/
   ```

   Move every `task_force_radio_win*.dll` and `TFAR_win*.dll` you see into the backup folder. A
   Linux TeamSpeak never loads them, but they hide which TFAR plugin is installed.

   **Expected:** the listing, run again, shows no `task_force_radio_win*.dll` and no
   `TFAR_win*.dll`.

3. Copy the plugin `.so` files, the sounds and the HRIR sphere into the TeamSpeak plugins folder.
   Both versions' sound files are byte-identical, so one `radio-sounds` folder serves both plugins,
   and both load the sphere from `tfar_hrir/IRC_1002_C.bin` in the plugins folder.

   ```bash
   cp target/dist/tfar_0_9/task_force_radio_linux_amd64.so target/dist/tfar_1_0/TFAR_linux_amd64.so ~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/
   cp -r target/dist/tfar_1_0/radio-sounds target/dist/tfar_1_0/tfar_hrir ~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/
   ls -R ~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/
   ```

   **Expected:** the plugins folder lists `task_force_radio_linux_amd64.so`, `TFAR_linux_amd64.so`,
   `tfar_hrir/` with `IRC_1002_C.bin` and `CREDITS.md`, and `radio-sounds/` with `on.wav`, `off.wav`
   and the `sw`, `lr`, `dd` and `ab` folders.

4. Install the extension DLL into the mod root of the mod you test. First move the original
   `task_force_radio_pipe_x64.dll` (and the 32-bit `task_force_radio_pipe.dll`, if present; 64-bit
   Arma never loads it) from that mod root into the backup folder, then copy the matching build
   (`tfar_1_0` for TFAR 1.0, `tfar_0_9` for TFAR 0.9.12).

   ```bash
   cp target/dist/tfar_1_0/task_force_radio_pipe_x64.dll "<installed mod root>/"
   ls -l "<installed mod root>"/task_force_radio_pipe*.dll target/dist/tfar_1_0/task_force_radio_pipe_x64.dll
   ```

   **Expected:** the mod root holds exactly one `task_force_radio_pipe_x64.dll`, the size of the
   one in the dist folder. A Steam Workshop update or file verification can restore the original;
   repeat this step after one.

5. Turn BattlEye off: in the Arma 3 Launcher, open Parameters, All Parameters, Advanced, and untick
   Enable BattlEye. BattlEye blocks extensions it has not whitelisted.

   **Expected:** the launcher shows BattlEye disabled; Arma's RPT log later shows
   `task_force_radio_pipe` loading without a BattlEye block message.

6. Start TeamSpeak and check the plugin list: Tools, Options, Addons, Plugins.

   ```bash
   flatpak run com.teamspeak.TeamSpeak3
   ```

   **Expected:** both plugins are listed and enabled: "Task Force Arrowhead Radio" (1.0) and "Task
   Force Arma 3 Radio" (0.9.12), with no load error in TeamSpeak's client log. A plugin that cannot
   find `tfar_hrir/IRC_1002_C.bin` says so in the chat tab and places voices with the simple ILD.

7. With TeamSpeak running, run the probe under Proton's wine, without Arma, to test the extension and
   the plugin alone. Build the probe, then run it in a scratch wine prefix (created on first use;
   Arma's own prefix stays untouched) with the DLL as its one argument; without further arguments it
   sends `TS_INFO\tPING` and `TS_INFO\tVERSION`.

   ```bash
   cargo build -p tfar_extension_probe --release --target x86_64-pc-windows-gnu
   WINEPREFIX="$HOME/.local/share/tfar-probe-prefix" "$HOME/.local/share/Steam/steamapps/common/Proton 10.0/files/bin/wine" target/x86_64-pc-windows-gnu/release/tfar_extension_probe.exe target/dist/tfar_1_0/task_force_radio_pipe_x64.dll
   ```

   **Expected:** the probe prints `library:`, `version:`, then for the call `TS_INFO\tPING` the line
   `reply: PONG`. With TeamSpeak closed the reply is the not-connected string of that version:
   `Not connected to TeamSpeak` (1.0) or `Pipe not opened from TS plugin` (0.9). Repeat with
   `target/dist/tfar_0_9/task_force_radio_pipe_x64.dll` for the other version.

8. Ask the plugin for a ping from Arma. Start Arma with the mod under Proton (BattlEye off), open
   the editor, place a unit, press Play, and run this in the debug console (Local Exec):

   ```text
   "task_force_radio_pipe" callExtension ("TS_INFO" + toString [9] + "PING")
   ```

   This is `"task_force_radio_pipe" callExtension "TS_INFO\tPING"` with the tab written out, since
   SQF strings have no `\t` escape.

   **Expected:** the console shows `"PONG"`. `"Not connected to TeamSpeak"` (1.0) or
   `"Pipe not opened from TS plugin"` (0.9) means the extension runs but no plugin answered.

## Verify

Steps 7 and 8 return `PONG` for the mod under test, and TeamSpeak's client log shows the plugin's
game-link connection. A two-client session in a serious-mode channel moves both clients, renames
them to their game names, and carries SW, LR and DD radio traffic; after the mission ends, the
plugin gives the nickname back, lifts its mutes and goes dormant (D25).

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| A plugin is missing from the list, or TeamSpeak shows two TFAR entries of one version | An old Windows DLL is still in the plugins folder, or a stale `.so` with another name | Step 2 again; keep exactly one `.so` per version |
| `Not connected to TeamSpeak` or `Pipe not opened from TS plugin` in step 7 or 8 | TeamSpeak is not running, the plugin is disabled, or the plugin cannot bind its port | Enable the plugin; check TeamSpeak's log for a port-in-use line (0.9: 47810 and 47811, 1.0: 47820 and 47821; see the [game link protocol](/documentation/architecture/game_link_protocol.md#ports)) |
| `callExtension` returns an empty string at once | BattlEye blocked the DLL, or the original DLL is still loaded | Step 5; restart Arma after replacing the DLL |
| The probe reports that it cannot load the DLL | The DLL was built for the wrong target, or a mingw runtime is missing | Rebuild with `cargo xtask dist`; the build links the C runtime statically, so the DLL imports only Windows system DLLs |
| Windows players do not hear our tangent sounds | The plugin file name does not give the TeamSpeak plugin name the originals accept | Keep the dist names `task_force_radio_linux_amd64.so` (0.9) and `TFAR_linux_amd64.so` (1.0); never rename the file |
| A Claude Code session in this repository blocks every tool call after a build | A host hook runs `target/debug/xtask ai guard`, which our xtask does not have (OF-012) | Build with `CARGO_TARGET_DIR=<repo>/target-dist` and read `target-dist/dist/` for `target/dist/` in the steps |

## Related documentation

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — ports, frames and every
  reply string this runbook checks.
- [Roadmap](/documentation/roadmap.md) — which milestone makes each step meaningful.
