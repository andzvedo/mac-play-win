#!/usr/bin/env bash
# Instala as ferramentas, o Wine e o Ponte neste Mac.
# Uso: ./scripts/install-local.sh [--dev|--web] [--skip-wine] [--help]

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="app"
INSTALL_WINE=1

if [[ -t 1 ]]; then
  BOLD="$(printf '\033[1m')"
  DIM="$(printf '\033[2m')"
  GREEN="$(printf '\033[32m')"
  YELLOW="$(printf '\033[33m')"
  RED="$(printf '\033[31m')"
  RESET="$(printf '\033[0m')"
else
  BOLD=""
  DIM=""
  GREEN=""
  YELLOW=""
  RED=""
  RESET=""
fi

log() {
  printf '%s\n' "$*"
}

ok() {
  printf '%s✓%s %s\n' "$GREEN" "$RESET" "$*"
}

warn() {
  printf '%s!%s %s\n' "$YELLOW" "$RESET" "$*"
}

fail() {
  printf '%s✗%s %s\n' "$RED" "$RESET" "$*" >&2
  exit 1
}

DEV_PORT=43123

free_dev_port() {
  local pids
  pids="$(lsof -nP -tiTCP:"$DEV_PORT" -sTCP:LISTEN 2>/dev/null || true)"
  [[ -n "$pids" ]] || return 0
  warn "A libertar a porta $DEV_PORT (Vite anterior)…"
  # shellcheck disable=SC2086
  kill $pids 2>/dev/null || true
  local n=0
  while [[ $n -lt 15 ]]; do
    pids="$(lsof -nP -tiTCP:"$DEV_PORT" -sTCP:LISTEN 2>/dev/null || true)"
    [[ -z "$pids" ]] && return 0
    sleep 0.2
    n=$((n + 1))
  done
  # shellcheck disable=SC2086
  kill -9 $pids 2>/dev/null || true
}

step() {
  printf '\n%s%s%s\n' "$BOLD" "$*" "$RESET"
}

usage() {
  cat <<'EOF'
Instalador local do Ponte — ferramentas, Wine e o app neste Mac.

Uso:
  ./scripts/install-local.sh
  ./scripts/install-local.sh --dev
  ./scripts/install-local.sh --web
  npm run install:local
  npm run install:local:dev

Opções:
  --dev         Depois das dependências, abre o Ponte em modo desenvolvimento
  --web         Só a interface no navegador (Iniciar fica simulado)
  --skip-wine   Não instala Wine durante o setup local
  --help        Mostra esta ajuda

O Ponte não baixa jogos. Precisa de um .exe que já esteja no disco.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dev)
      MODE="dev"
      ;;
    --web)
      MODE="web"
      ;;
    --skip-wine)
      INSTALL_WINE=0
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "Opção desconhecida: $1 (use --help)"
      ;;
  esac
  shift
done

node_major() {
  node -p "parseInt(process.versions.node.split('.')[0], 10)" 2>/dev/null || echo 0
}

load_existing_tools() {
  if [[ -x /opt/homebrew/bin/brew ]]; then
    eval "$(/opt/homebrew/bin/brew shellenv)"
  elif [[ -x /usr/local/bin/brew ]]; then
    eval "$(/usr/local/bin/brew shellenv)"
  fi

  if [[ -s "${HOME}/.nvm/nvm.sh" ]]; then
    # shellcheck disable=SC1091
    . "${HOME}/.nvm/nvm.sh"
  fi

  if [[ -s "${HOME}/.cargo/env" ]]; then
    # shellcheck disable=SC1091
    . "${HOME}/.cargo/env"
  fi
}

ensure_macos() {
  [[ "$(uname -s)" == "Darwin" ]] || fail "Este instalador é para macOS."
  if [[ "$(id -u)" -eq 0 ]]; then
    fail "Não corra o instalador como root. Use a sua conta normal."
  fi
}

ensure_clt() {
  step "Command Line Tools"
  if xcode-select -p >/dev/null 2>&1 && command -v clang >/dev/null 2>&1; then
    ok "Já instaladas ($(xcode-select -p))"
    return
  fi
  warn "Vai abrir o instalador da Apple. Conclua e volte aqui."
  xcode-select --install || true
  local waited=0
  while ! { xcode-select -p >/dev/null 2>&1 && command -v clang >/dev/null 2>&1; }; do
    if [[ "$waited" -ge 1800 ]]; then
      fail "As Command Line Tools ainda não estão prontas. Corra de novo depois de instalar."
    fi
    sleep 5
    waited=$((waited + 5))
  done
  ok "Command Line Tools prontas"
}

ensure_rosetta() {
  step "Rosetta"
  if [[ "$(uname -m)" != "arm64" ]]; then
    ok "Não é Apple Silicon — Rosetta não é necessário"
    return
  fi
  if /usr/bin/pgrep -q oahd; then
    ok "Rosetta já está instalado"
    return
  fi
  sudo softwareupdate --install-rosetta --agree-to-license
  ok "Rosetta instalado"
}

ensure_brew() {
  step "Homebrew"
  if command -v brew >/dev/null 2>&1; then
    ok "Homebrew em $(command -v brew)"
    return
  fi
  /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
  if [[ -x /opt/homebrew/bin/brew ]]; then
    eval "$(/opt/homebrew/bin/brew shellenv)"
  elif [[ -x /usr/local/bin/brew ]]; then
    eval "$(/usr/local/bin/brew shellenv)"
  fi
  command -v brew >/dev/null 2>&1 || fail "Homebrew instalou, mas o brew não entrou no PATH. Feche o Terminal e corra o instalador outra vez."
  ok "Homebrew instalado"
}

ensure_node() {
  step "Node.js 22+"
  if command -v node >/dev/null 2>&1 && [[ "$(node_major)" -ge 22 ]]; then
    ok "Node $(node -v)"
    return
  fi
  command -v brew >/dev/null 2>&1 || fail "É preciso Homebrew para instalar o Node."
  brew install node
  hash -r
  command -v node >/dev/null 2>&1 || fail "Node não ficou disponível depois do brew install."
  if [[ "$(node_major)" -lt 22 ]]; then
    fail "O Node instalado é $(node -v). O Ponte pede 22 ou superior."
  fi
  ok "Node $(node -v)"
}

ensure_rust() {
  if [[ "$MODE" == "web" ]]; then
    return
  fi
  step "Rust 1.88"
  if ! command -v rustup >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.88.0
    # shellcheck disable=SC1091
    . "${HOME}/.cargo/env"
  fi
  rustup toolchain install 1.88.0 --profile minimal
  ok "Rust $(rustc +1.88.0 --version 2>/dev/null || rustc --version)"
}

find_wine() {
  local candidate
  for candidate in \
    /opt/homebrew/bin/wine64 \
    /opt/homebrew/bin/wine \
    /usr/local/bin/wine64 \
    /usr/local/bin/wine \
    "/Applications/Wine Stable.app/Contents/Resources/wine/bin/wine64" \
    "/Applications/Wine Stable.app/Contents/Resources/wine/bin/wine" \
    "/Applications/Wine Staging.app/Contents/Resources/wine/bin/wine" \
    /usr/local/opt/game-porting-toolkit/bin/wine64 \
    /opt/homebrew/opt/game-porting-toolkit/bin/wine64 \
    /opt/homebrew/opt/wine-crossover/bin/wine64 \
    "/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine64" \
    "/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine"
  do
    if [[ -x "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
  return 1
}

ensure_wine() {
  if [[ "$MODE" == "web" || "$INSTALL_WINE" -eq 0 ]]; then
    return
  fi
  step "Wine"
  local current
  if current="$(find_wine)"; then
    ok "Já existe: $current"
    return
  fi
  command -v brew >/dev/null 2>&1 || fail "É preciso Homebrew para instalar o Wine."
  if brew install --cask --no-quarantine wine@staging; then
    ok "Wine Staging instalado (melhor DirectX em Apple Silicon)"
    return
  fi
  warn "Não deu para instalar o Wine Staging pelo Homebrew."
  warn "O Ponte pode instalar o Wine Staging gerenciado ao preparar o primeiro jogo."
  warn "GPTK, Gcenx e CrossOver instalados manualmente também são detetados."
}

install_npm_deps() {
  step "Dependências npm"
  cd "$ROOT"
  if [[ -f package-lock.json ]]; then
    npm ci
  else
    npm install
  fi
  ok "node_modules pronto"
}

install_app() {
  local app_src dest
  app_src="$ROOT/src-tauri/target/release/bundle/macos/Ponte.app"
  [[ -d "$app_src" ]] || fail "O build não gerou Ponte.app em $app_src"

  if [[ -w /Applications ]]; then
    dest="/Applications/Ponte.app"
  else
    mkdir -p "${HOME}/Applications"
    dest="${HOME}/Applications/Ponte.app"
  fi

  rm -rf "$dest"
  ditto "$app_src" "$dest"
  ok "App em $dest"
  open "$dest"
}

run_mode() {
  cd "$ROOT"
  case "$MODE" in
    web)
      step "Interface no navegador"
      log "${DIM}Iniciar fica simulado neste modo. Ctrl+C para sair.${RESET}"
      free_dev_port
      npm start
      ;;
    dev)
      step "Ponte em desenvolvimento"
      log "${DIM}A primeira compilação demora. Ctrl+C para sair.${RESET}"
      free_dev_port
      npm run tauri:dev
      ;;
    app)
      step "Compilar e instalar o Ponte"
      log "${DIM}A primeira compilação nativa pode levar vários minutos.${RESET}"
      npm run tauri:build:app
      install_app
      if current="$(find_wine)"; then
        ok "Wine detectado: $current"
      else
        warn "Nenhum wine64 encontrado. Em Ajustes, use Escolher para apontar o binário."
      fi
      log ""
      ok "Pode adicionar um .exe local e iniciar."
      ;;
    *)
      fail "Modo interno inválido: $MODE"
      ;;
  esac
}

if [[ "${BASH_SOURCE[0]}" != "$0" ]]; then
  return 0
fi

printf '%sPonte — instalador local%s\n' "$BOLD" "$RESET"
log "Pasta: $ROOT"

ensure_macos
load_existing_tools
ensure_clt
ensure_rosetta
ensure_brew
load_existing_tools
ensure_node
ensure_rust
install_npm_deps
ensure_wine
run_mode
