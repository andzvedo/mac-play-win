#!/bin/bash
# Duplo clique no Finder para instalar e abrir o Ponte neste Mac.
cd "$(dirname "$0")"
./scripts/install-local.sh
status=$?
echo
read -r -p "Prima Enter para fechar…"
exit "$status"
