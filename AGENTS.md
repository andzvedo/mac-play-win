## Learned User Preferences

- Communicate in Portuguese.
- Prefer a one-command local installer (`Instalar.command`) and a one-click launch (`Abrir.command`) over manual toolchain steps.
- Ask the agent to verify locally (native app, tests, Wine) rather than only give instructions.
- Test with games already on disk; the app must not download games.
- Run npm/tauri commands from the project folder, not from `~`.
- Add games by selecting the game folder only; the app finds the `.exe` and fills metadata, engine, and backend.
- Auto-install missing runtimes once; skip reinstall when already present or cached.
- Keep vertical scroll user-controlled; do not auto-scroll or lock it while analyzing a game.

## Learned Workspace Facts

- Ponte is a Vite + Tauri launcher for local Windows `.exe` files on Apple Silicon Macs; it is not a store or emulator and does not download games.
- `npm start` serves the UI at http://127.0.0.1:43123 with simulated launch; real Wine launch requires `npm run tauri:dev` or `Abrir.command` (which frees port 43123 if it is already in use).
- Toolchain: Node 22+, Rust 1.88+, Xcode Command Line Tools; Rosetta on Apple Silicon.
- Local installer lives at `scripts/install-local.sh` and `Instalar.command`; `Abrir.command` launches the app.
- WineHQ Stable 11 exposes `wine`, not `wine64`; detection accepts both and must require 64-bit `kernel32` (Mythic Engine Wine is incomplete).
- WineHQ Stable + WineD3D cannot provide D3D11 on Apple Silicon; use GPTK + D3DMetal or Wine Staging (Gcenx) + DXMT/DXVK.
- macOS native file dialogs must be async Tauri commands with `set_parent`, or the UI freezes.
- Tauri icon PNGs must be RGBA or `tauri:dev` fails to build.
- App data lives in `~/Library/Application Support/Ponte/` (`library.json`, prefixes, and `cache/vcrun2015-x64`).
- Plan T1–T37 is implemented; Hydra, stores, torrents, auto-GPTK/CrossOver install, and kernel anti-cheat stay out of scope. The app can auto-install Wine Staging (Homebrew), a Wine prefix, DXVK-macOS, and VC++ 2015–2022.
- Games are added by folder; opening a game that is not ready detects missing pieces and prepares them automatically.
- Game prefixes may need the official Visual C++ 2015–2022 x64 redistributable; it is cached once and copied into prefixes, and detection must not treat a prefix as complete without the real install.
