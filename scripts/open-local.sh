#!/usr/bin/env bash
# Abre o Ponte sem precisar de comandos no Terminal.
# Prefere o .app instalado; se não existir, sobe o modo desenvolvimento.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

load_tools() {
  if [[ -x /opt/homebrew/bin/brew ]]; then
    eval "$(/opt/homebrew/bin/brew shellenv)"
  elif [[ -x /usr/local/bin/brew ]]; then
    eval "$(/usr/local/bin/brew shellenv)"
  fi

  if [[ -s "${HOME}/.cargo/env" ]]; then
    # shellcheck disable=SC1091
    . "${HOME}/.cargo/env"
  fi
}

app_candidates() {
  printf '%s\n' \
    "/Applications/Ponte.app" \
    "${HOME}/Applications/Ponte.app" \
    "${ROOT}/src-tauri/target/release/bundle/macos/Ponte.app"
}

bring_to_front() {
  if pgrep -xq ponte 2>/dev/null; then
    open -a Ponte 2>/dev/null && return 0
  fi
  return 1
}

open_installed_app() {
  local dest
  while IFS= read -r dest; do
    if [[ -d "$dest" ]]; then
      open "$dest"
      return 0
    fi
  done < <(app_candidates)
  return 1
}

if bring_to_front; then
  exit 0
fi

if open_installed_app; then
  exit 0
fi

# Vite/tauri:dev usam esta porta em exclusivo (strictPort).
DEV_PORT=43123

free_dev_port() {
  local pids
  pids="$(lsof -nP -tiTCP:"$DEV_PORT" -sTCP:LISTEN 2>/dev/null || true)"
  [[ -n "$pids" ]] || return 0
  echo "A libertar a porta $DEV_PORT (Vite anterior)…"
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

load_tools
cd "$ROOT"

if [[ ! -d node_modules ]]; then
  echo "A instalar dependências (só na primeira vez)…"
  if [[ -f package-lock.json ]]; then
    npm ci
  else
    npm install
  fi
fi

echo "A abrir o Ponte…"
echo "Pode fechar esta janela depois que o app aparecer. Ctrl+C também encerra."
free_dev_port
npm run tauri:dev
