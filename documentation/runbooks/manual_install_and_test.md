**Status:** live

# Install the Rust plugins and extensions by hand and test them

Builds both mods' artifacts, installs the plugins into the Flatpak TeamSpeak and the extension DLL
into an installed mod, and checks the link from TeamSpeak, from Proton's wine and from Arma. No
script copies anything into TeamSpeak or a mod folder (decision D13); every copy below is done by
the owner. Run the build from the repository root,
`/run/media/system/Disk_2/Projects/Taskforceradio`.

Until M3 the plugins and extensions are stubs: the TeamSpeak steps (1–3, 6) work from M2, the Arma
and probe steps (4, 5, 7, 8) from M3, and a real game from M6 (0.9) and M7 (1.0).

## Prerequisites

- The toolchain from `rust-toolchain.toml` with the `x86_64-pc-windows-gnu` target, and
  `x86_64-w64-mingw32-gcc` on `PATH`.
- TeamSpeak 3.6.2 as the Flatpak `com.teamspeak.TeamSpeak3`; its plugins folder is
  `~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/`.
- Arma 3 under Proton (Steam app 107410) with the mod to test installed: TFAR 1.0 or TFAR 0.9.12.
  Its mod root is the folder that holds mod.cpp and the addons folder (for a Workshop install, under
  Steam's workshop content folder for 107410).
- A backup folder outside the mod and TeamSpeak folders, for the original files this runbook
  replaces.

## Steps

1. Build every artifact.

   ```bash
   cargo xtask dist
   ```

   **Expected:** the dist folder under target holds a tfar_1_0 and a tfar_0_9 folder, each with the
   plugin `.so` for Linux (`task_force_radio_linux_amd64.so` in tfar_0_9, `TFAR_linux_amd64.so`
   in tfar_1_0; TeamSpeak takes the plugin name from the file name, and the originals accept plugin
   commands only from those names), `task_force_radio_pipe_x64.dll` for Windows, and a radio-sounds
   folder.

2. Close TeamSpeak, then remove the old Windows plugin files by hand.

   ```bash
   flatpak kill com.teamspeak.TeamSpeak3
   ls ~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/
   ```

   Move every `task_force_radio_win*.dll` and `TFAR_win*.dll` you see into the backup folder.

   **Expected:** the listing shows no `task_force_radio_win*.dll` and no `TFAR_win*.dll` when run
   again.

3. Copy the plugin `.so` files and the sounds into the TeamSpeak plugins folder. Both versions'
   sound files are byte-identical, so one `radio-sounds` folder serves both plugins.

   ```bash
   cp target/dist/tfar_1_0/*.so target/dist/tfar_0_9/*.so ~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/
   cp -r target/dist/tfar_1_0/radio-sounds ~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/
   ```

   **Expected:** the plugins folder lists both `.so` files and the radio-sounds folder with `on.wav`,
   `off.wav` and the `sw`, `lr`, `dd` and `ab` folders.

4. Install the extension DLL into the mod root of the mod you test. First move the original
   `task_force_radio_pipe_x64.dll` (and the 32-bit `task_force_radio_pipe.dll`, if present; 64-bit
   Arma never loads it) from that mod root into the backup folder, then copy the matching build
   (`tfar_1_0` for TFAR 1.0, `tfar_0_9` for TFAR 0.9.12).

   ```bash
   cp target/dist/tfar_1_0/task_force_radio_pipe_x64.dll "<mod root>/"
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
   Force Arma 3 Radio" (0.9.12), with no load error in TeamSpeak's client log.

7. Ask the plugin for a ping from Arma. Start Arma with the mod under Proton, open the editor, place
   a unit, press Play, and run this in the debug console (Local Exec):

   ```text
   "task_force_radio_pipe" callExtension ("TS_INFO" + toString [9] + "PING")
   ```

   **Expected:** the console shows `"PONG"`. `"Not connected to TeamSpeak"` (1.0) or
   `"Pipe not opened from TS plugin"` (0.9) means the extension runs but no plugin answered.

8. Run the probe under Proton's wine, without Arma, to test the extension and the plugin alone.
   Build it, then run it in Arma's Proton prefix with the DLL and the call text (its arguments are
   the ones `tfar_extension_probe.exe --help` prints).

   ```bash
   cargo build -p tfar_extension_probe --release --target x86_64-pc-windows-gnu
   WINEPREFIX="$HOME/.steam/steam/steamapps/compatdata/107410/pfx" "<Proton folder>/files/bin/wine" target/x86_64-pc-windows-gnu/release/tfar_extension_probe.exe target/dist/tfar_1_0/task_force_radio_pipe_x64.dll $'TS_INFO\tPING'
   ```

   **Expected:** the probe prints `PONG` while TeamSpeak runs with the plugin, and the not-connected
   string of that version when TeamSpeak is closed.

## Verify

Step 7 returns `PONG` for the mod under test, and TeamSpeak's client log shows the plugin's
game-link connection. From M6 and M7, a two-client session in a serious-mode channel moves both
clients, renames them to their game names, and carries SW, LR and DD radio traffic.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| A plugin is missing from the list, or TeamSpeak shows two TFAR entries of one version | An old Windows DLL is still in the plugins folder, or a stale `.so` with another name | Step 2 again; keep exactly one `.so` per version |
| `Not connected to TeamSpeak` or `Pipe not opened from TS plugin` in step 7 | TeamSpeak is not running, the plugin is disabled, or the plugin cannot bind its port | Enable the plugin; check TeamSpeak's log for a port-in-use line (ports 47810 and 47820, see the [game link protocol](/documentation/architecture/game_link_protocol.md#ports)) |
| `callExtension` returns an empty string at once | BattlEye blocked the DLL, or the original DLL is still loaded | Step 5; restart Arma after replacing the DLL |
| The probe reports that it cannot load the DLL | The DLL was built for the wrong target, or a mingw runtime is missing | Rebuild with `cargo xtask dist`; the build links the C runtime statically |
| Windows players do not hear our tangent sounds | The plugin file name does not give the TeamSpeak plugin name the originals accept | Keep the dist names `task_force_radio_linux_amd64.so` (0.9) and `TFAR_linux_amd64.so` (1.0); never rename the file |

## Related documentation

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — ports, frames and every
  reply string this runbook checks.
- [Roadmap](/documentation/roadmap.md) — which milestone makes each step meaningful.
