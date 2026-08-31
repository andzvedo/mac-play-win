#!/bin/bash
# Duplo clique no Finder para abrir o Ponte.
SOURCE="$0"
while [[ -L "$SOURCE" ]]; do
  DIR="$(cd "$(dirname "$SOURCE")" && pwd)"
  SOURCE="$(readlink "$SOURCE")"
  [[ "$SOURCE" != /* ]] && SOURCE="$DIR/$SOURCE"
done
ROOT="$(cd "$(dirname "$SOURCE")" && pwd)"
"$ROOT/scripts/open-local.sh"
status=$?
if [[ "$status" -ne 0 ]]; then
  echo
  read -r -p "Falhou. Prima Enter para fechar…"
fi
exit "$status"
