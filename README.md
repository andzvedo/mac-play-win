# Ponte

Launcher para **jogos Windows que você já tem no disco**, no MacBook Apple Silicon.

Não é emulador, não é loja e não baixa jogos. O Ponte aponta a pasta de um jogo Windows que já está no disco, encontra o `.exe`, cria um prefixo Wine e dispara o `wine64` que estiver no seu Mac.

## Rodar no Mac

O instalador local trata de Command Line Tools, Rosetta, Homebrew, Node 22+, Rust 1.88, Wine e do app:

```bash
./scripts/install-local.sh
```

No Finder, também pode abrir `Instalar.command`. A primeira compilação demora.

- `./scripts/install-local.sh --dev` — janela nativa em desenvolvimento
- `./scripts/install-local.sh --web` — só a interface no navegador
- `./scripts/install-local.sh --skip-wine` — não instala o Wine

Ou abra esta pasta no Terminal e faça os passos à mão.

### 1. Node 22+

```bash
node -v
```

Se não tiver: [nodejs.org](https://nodejs.org/) ou `brew install node`.

### 2. Dependências

```bash
npm install
```

### 3. Escolha o modo

**Só a interface** (navegador; o botão Iniciar é simulado):

```bash
npm start
```

Abre [http://127.0.0.1:43123](http://127.0.0.1:43123).

**Jogo de verdade** (janela nativa + Wine):

```bash
xcode-select --install
softwareupdate --install-rosetta --agree-to-license
```

Instale Rust se ainda não tiver:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

O projeto pede Rust **1.88+** (`rust-toolchain.toml`). Depois:

```bash
npm run tauri:dev
```

A primeira compilação demora. Na janela Ponte: **Ajustes** → confira se o `wine64` foi encontrado (ou **Escolher** o binário) → **Adicionar jogo** e escolha a pasta do título no disco → **Iniciar**.

### 4. Wine (obrigatório para jogar)

Na ficha do jogo, **Instalar o que falta** (e o próprio **Iniciar**) analisa a pasta, encontra o `.exe` e instala o que der: Wine Staging via Homebrew se não houver `wine64`, prefixo, DXVK-macOS no prefixo (DirectX 11) e Visual C++ 2015–2022. O GPTK e o CrossOver continuam manuais:

- [Game Porting Toolkit](https://developer.apple.com/games/game-porting-toolkit/) ou [Gcenx/game-porting-toolkit](https://github.com/Gcenx/game-porting-toolkit)
- [Wine Staging Gcenx](https://github.com/Gcenx/macOS_Wine_builds) (também via `brew install --cask --no-quarantine wine@staging`)
- [Wine CX](https://github.com/Gcenx/winecx) / CrossOver

### App para a pasta Aplicativos

```bash
npm run tauri:build
```

O `.app` sai em `src-tauri/target/release/bundle/macos/`.

### Checagens

```bash
npm run check
```

## O que não cobre

Hydra, torrents, lojas Steam/Epic, instalar GPTK/D3DMetal por você, anti-cheat de kernel. Trocar o motor pode ajudar; não é garantia de que o título abre.

## Licença

MIT
