# Game link

The link between the Arma extensions (Windows DLLs under Proton) and the native TeamSpeak plugins:

- [`tfar_game_link`](/crates/game_link/tfar_game_link/README.md) — the frames, the plugin's TCP
  server and the extension's client; builds for Linux and Windows.
- [`tfar_arma_extension_abi`](/crates/game_link/tfar_arma_extension_abi/README.md) — the
  `RVExtension` exports and the Win32 focus query; Windows-only, on the unsafe allow-list, and the
  only crate with `windows-sys`.

```text
Arma (Proton) -- callExtension --> extension DLL -- TCP 127.0.0.1 --> TeamSpeak plugin (.so)
                                   tfar_game_link::client             tfar_game_link::server
```

Game link crates depend only on foundation and each other; the version libraries may use
`tfar_game_link` but not the Arma ABI crate (`cargo xtask verify crate-tiers`).
