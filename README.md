# Ponte

Launcher para **jogos Windows que você já tem no disco**, no MacBook Apple Silicon. O Ponte cria um prefixo Wine por jogo, deixa você escolher o motor (GPTK, Wine Staging Gcenx, winecx, CrossOver ou um `wine64` manual) e dispara o `.exe`.

Não é emulador de Windows, não é loja e não baixa jogos. Também não garante que um título específico abra.

## O que cobre

- Biblioteca de `.exe` locais
- Prefixo Wine isolado por jogo
- Seletor de motor e de backend DirectX (D3DMetal, DXMT, DXVK-macOS, WineD3D)
- Iniciar / parar e log da saída do Wine
- Ajustes: caminho manual do `wine64`, pasta de prefixos, HUD Metal

## O que não cobre

- Hydra, torrents, scrapers de repacks
- Lojas (Steam, Epic, GOG) como cliente
- Instalar o Game Porting Toolkit por você
- Anti-cheat de kernel (Easy Anti-Cheat, BattlEye, Vanguard)
- Redistribuir D3DMetal (licença da Apple)

Nem CrossOver pago abre todos os jogos. Anti-cheat, DRM e DirectX incompleto ainda barram títulos. Trocar o motor pode ajudar; o log mostra a falha.

## Pré-requisito no Mac

Pelo menos um `wine64`:

- [Game Porting Toolkit](https://developer.apple.com/games/game-porting-toolkit/) (conta Apple Developer), ou o build [Gcenx](https://github.com/Gcenx/game-porting-toolkit)
- ou [Wine Staging Gcenx](https://github.com/Gcenx/macOS_Wine_builds)
- ou [Wine CX](https://github.com/Gcenx/winecx) / CrossOver já instalado

O Ponte **não empacota** Wine nem D3DMetal. Depois de instalar, abra Ajustes e confira se o binário foi detectado (ou cole o caminho).

## Como rodar

```bash
npm install
```

Só a interface (navegador, launch simulado):

```bash
npm run dev
```

Abre em `http://127.0.0.1:43123`. Neste modo a detecção de motores é um mock e o “Iniciar” não dispara Wine.

No Mac, com o jogo de verdade:

```bash
npm run tauri dev
```

Requer [pré-requisitos do Tauri](https://v2.tauri.app/start/prerequisites/) (Xcode CLT, Rust). O projeto pede Rust **1.88+** (`rust-toolchain.toml`).

```bash
npm test
npm run build
```

## Motores OSS

O Ponte só orquestra o que a comunidade já mantém:

| Camada | Projeto |
| --- | --- |
| Wine no Mac | [Gcenx/macOS_Wine_builds](https://github.com/Gcenx/macOS_Wine_builds) |
| Wine CX | [Gcenx/winecx](https://github.com/Gcenx/winecx) |
| DX12/11 → Metal | Apple GPTK / D3DMetal |
| DX10/11 → Metal | [DXMT](https://github.com/3Shain/dxmt) |
| DX10/11 → Vulkan | [DXVK-macOS](https://github.com/Gcenx/DXVK-macOS) |

Heroic e Mythic já fazem loja + sideload. O Ponte é o fluxo “tenho o `.exe` neste disco”.

## Licença

MIT
