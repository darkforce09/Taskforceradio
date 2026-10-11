# Tools

The repository tooling; nothing here ships to a player.

| Tool | What it is |
|---|---|
| [xtask](/tools/xtask/README.md) | `cargo xtask`: the gates, the `ci-local` lane, dist, the reference manifest |
| [tfar_ts3_mock](/tools/tfar_ts3_mock/README.md) | Loads a built plugin `.so` and plays TeamSpeak and the game side of the link, so plugin behaviour is tested without TeamSpeak or Arma |
| [tfar_extension_probe](/tools/tfar_extension_probe/README.md) | A Windows executable that stands in for Arma under Proton's wine and calls the extension DLL |
| [tfar_golden_vectors](/tools/tfar_golden_vectors/README.md) | Builds the original C/C++ DSP code as an oracle the audio tests compare against; the only C/C++ the repository compiles |

```bash
cargo build -p tfar_extension_probe --target x86_64-pc-windows-gnu   # tfar_extension_probe.exe
cargo test -p tfar_ts3_mock -p tfar_golden_vectors -p tfar_extension_probe
```

Tools may depend on library crates under `crates/`, never on the four deliverables
(`cargo xtask verify crate-tiers`); `cc` stays in `tfar_golden_vectors` and `libloading` in the
mock and the probe (`cargo xtask verify dependency-firewalls`).
