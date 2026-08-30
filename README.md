# Ponte

Launcher para **jogos Windows que você já tem no disco**, no MacBook Apple Silicon.

Não é emulador, não é loja e não baixa jogos. O Ponte aponta um `.exe` local, cria um prefixo Wine e dispara o `wine64` que estiver no seu Mac.

## Rodar no Mac

Abra esta pasta do projeto no Terminal (ou no Cursor → Terminal).

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

A primeira compilação demora. Na janela Ponte: **Ajustes** → confira se o `wine64` foi encontrado (ou **Escolher** o binário) → **Adicionar jogo** com o `.exe` no disco → **Iniciar**.

### 4. Wine (obrigatório para jogar)

O Ponte **não instala** Wine. Precisa de pelo menos um `wine64`:

- [Game Porting Toolkit](https://developer.apple.com/games/game-porting-toolkit/) ou [Gcenx/game-porting-toolkit](https://github.com/Gcenx/game-porting-toolkit)
- [Wine Staging Gcenx](https://github.com/Gcenx/macOS_Wine_builds)
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

Hydra, torrents, lojas Steam/Epic, instalar GPTK por você, anti-cheat de kernel. Trocar o motor pode ajudar; não é garantia de que o título abre.

## Licença

MIT
