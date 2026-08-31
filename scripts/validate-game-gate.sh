#!/usr/bin/env bash
# Valida uma fixture Windows já existente no disco e produz evidência redigida.
# Este script não faz download, instalação, alteração ou remoção da fixture.

set -euo pipefail
set -C
umask 077

usage() {
  cat <<'EOF'
Uso:
  scripts/validate-game-gate.sh PASTA_ABSOLUTA \
    --game-name NOME \
    --version VERSAO \
    --origin URL \
    --output EVIDENCIA_ABSOLUTA \
    [--executable EXECUTAVEL_ABSOLUTO]

Exemplo:
  scripts/validate-game-gate.sh \
    "/Users/andre/Downloads/Ponte Gate/The Powder Toy" \
    --game-name "The Powder Toy" \
    --version "<versão baixada>" \
    --origin "https://powdertoy.co.uk/" \
    --output "/Users/andre/Downloads/Ponte Gate/evidence/the-powder-toy.md"

Regras:
  - A pasta, o executável opcional e a saída devem ser caminhos absolutos.
  - A saída precisa estar numa pasta já existente, fora da pasta do jogo, e não
    pode existir ainda (incluindo como link simbólico quebrado).
  - Sem --executable, a validação só escolhe automaticamente quando encontra
    exatamente um arquivo .exe na pasta e nas subpastas.
  - O script nunca baixa, instala, move, apaga ou altera arquivos do jogo.

Códigos de saída:
  0  executável encontrado e reconhecido como PE Windows
  1  executável encontrado, mas a validação falhou
  2  uso inválido, ambiente incompatível ou pré-condição ausente
EOF
}

fail() {
  printf 'ERRO: %s\n' "$*" >&2
  exit 2
}

require_absolute_path() {
  case "$1" in
    /*) ;;
    *) fail "caminho não absoluto: $1" ;;
  esac
}

safe_text() {
  # Evita que metadados fornecidos pelo operador quebrem a estrutura Markdown.
  printf '%s' "$1" | tr '\r\n' '  ' | sed 's/`/\\`/g'
}

if [[ "$(uname -s)" != "Darwin" ]]; then
  fail "este validador requer macOS"
fi

command -v find >/dev/null 2>&1 || fail "find não está disponível"
command -v file >/dev/null 2>&1 || fail "file não está disponível"
command -v shasum >/dev/null 2>&1 || fail "shasum não está disponível"
command -v stat >/dev/null 2>&1 || fail "stat não está disponível"
command -v sw_vers >/dev/null 2>&1 || fail "sw_vers não está disponível"

[[ $# -ge 1 ]] || { usage >&2; exit 2; }

case "$1" in
  --help|-h)
    usage
    exit 0
    ;;
esac

GAME_INPUT="$1"
shift
GAME_NAME_INPUT=""
VERSION_INPUT=""
ORIGIN_INPUT=""
OUTPUT_INPUT=""
EXECUTABLE_INPUT=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --game-name)
      [[ $# -ge 2 ]] || fail "--game-name requer um valor"
      GAME_NAME_INPUT="$2"
      shift 2
      ;;
    --version)
      [[ $# -ge 2 ]] || fail "--version requer um valor"
      VERSION_INPUT="$2"
      shift 2
      ;;
    --origin)
      [[ $# -ge 2 ]] || fail "--origin requer um valor"
      ORIGIN_INPUT="$2"
      shift 2
      ;;
    --output)
      [[ $# -ge 2 ]] || fail "--output requer um caminho"
      OUTPUT_INPUT="$2"
      shift 2
      ;;
    --executable)
      [[ $# -ge 2 ]] || fail "--executable requer um caminho"
      EXECUTABLE_INPUT="$2"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "opção desconhecida: $1"
      ;;
  esac
done

require_absolute_path "$GAME_INPUT"
[[ -n "$VERSION_INPUT" ]] || fail "informe --version para a evidência"
[[ -n "$ORIGIN_INPUT" ]] || fail "informe --origin para a evidência"
[[ -n "$OUTPUT_INPUT" ]] || fail "informe --output para a evidência"
require_absolute_path "$OUTPUT_INPUT"

case "$ORIGIN_INPUT" in
  http://*|https://*) ;;
  *) fail "--origin deve ser uma URL http:// ou https://" ;;
esac

[[ -d "$GAME_INPUT" ]] || fail "a pasta da fixture não existe: $GAME_INPUT"
GAME_DIR="$(cd "$GAME_INPUT" && pwd -P)" || fail "não foi possível resolver a pasta da fixture"

OUTPUT_PARENT_INPUT="$(dirname "$OUTPUT_INPUT")"
OUTPUT_BASENAME="$(basename "$OUTPUT_INPUT")"
[[ -n "$OUTPUT_BASENAME" && "$OUTPUT_BASENAME" != "." && "$OUTPUT_BASENAME" != "/" ]] || fail "nome de saída inválido"
[[ "$OUTPUT_INPUT" != */ ]] || fail "a saída precisa ser um arquivo, não uma pasta"
[[ -d "$OUTPUT_PARENT_INPUT" ]] || fail "a pasta de saída não existe: $OUTPUT_PARENT_INPUT"
[[ -w "$OUTPUT_PARENT_INPUT" ]] || fail "a pasta de saída não permite escrita: $OUTPUT_PARENT_INPUT"
OUTPUT_PARENT="$(cd "$OUTPUT_PARENT_INPUT" && pwd -P)" || fail "não foi possível resolver a pasta de saída"
OUTPUT_PATH="$OUTPUT_PARENT/$OUTPUT_BASENAME"
[[ ! -e "$OUTPUT_PATH" && ! -L "$OUTPUT_PATH" ]] || fail "a saída já existe; escolha um caminho novo: $OUTPUT_PATH"
case "$OUTPUT_PATH" in
  "$GAME_DIR"/*) fail "a saída precisa ficar fora da pasta da fixture" ;;
esac

GAME_NAME="$GAME_NAME_INPUT"
if [[ -z "$GAME_NAME" ]]; then
  GAME_NAME="$(basename "$GAME_DIR")"
fi

if [[ -n "$EXECUTABLE_INPUT" ]]; then
  require_absolute_path "$EXECUTABLE_INPUT"
  [[ -f "$EXECUTABLE_INPUT" ]] || fail "o executável informado não existe: $EXECUTABLE_INPUT"
  [[ ! -L "$EXECUTABLE_INPUT" ]] || fail "o executável informado não pode ser um link simbólico"
  EXECUTABLE_PARENT="$(cd "$(dirname "$EXECUTABLE_INPUT")" && pwd -P)" || fail "não foi possível resolver o executável"
  EXECUTABLE_PATH="$EXECUTABLE_PARENT/$(basename "$EXECUTABLE_INPUT")"
  case "$EXECUTABLE_PATH" in
    "$GAME_DIR"/*) ;;
    *) fail "o executável precisa estar dentro da pasta da fixture" ;;
  esac
  case "$EXECUTABLE_PATH" in
    *.[eE][xX][eE]) ;;
    *) fail "o caminho informado não termina em .exe: $EXECUTABLE_PATH" ;;
  esac
else
  CANDIDATES=()
  CANDIDATE_COUNT=0
  while IFS= read -r -d '' CANDIDATE; do
    CANDIDATES[$CANDIDATE_COUNT]="$CANDIDATE"
    CANDIDATE_COUNT=$((CANDIDATE_COUNT + 1))
  done < <(find -E "$GAME_DIR" -type f -iregex '.*[.]exe' -print0 | sort -z)

  if [[ "$CANDIDATE_COUNT" -eq 0 ]]; then
    fail "nenhum arquivo .exe foi encontrado dentro da pasta da fixture"
  elif [[ "$CANDIDATE_COUNT" -gt 1 ]]; then
    printf 'ERRO: foram encontrados %s arquivos .exe; escolha um com --executable:\n' "$CANDIDATE_COUNT" >&2
    for CANDIDATE in "${CANDIDATES[@]}"; do
      printf '  %s\n' "${CANDIDATE#"$GAME_DIR"/}" >&2
    done
    exit 2
  fi
  EXECUTABLE_PATH="${CANDIDATES[0]}"
fi

[[ -f "$EXECUTABLE_PATH" ]] || fail "o executável selecionado não é um arquivo regular"
PE_TYPE_RAW="$(file -b "$EXECUTABLE_PATH")" || fail "file não conseguiu analisar o executável"
PE_TYPE="$(safe_text "$PE_TYPE_RAW")"
SIZE_BYTES="$(stat -f '%z' "$EXECUTABLE_PATH")" || fail "stat não conseguiu calcular o tamanho"
SHA256="$(shasum -a 256 "$EXECUTABLE_PATH" | awk '{print $1}')" || fail "shasum não conseguiu calcular o hash"
[[ "$SHA256" =~ ^[[:xdigit:]]{64}$ ]] || fail "SHA-256 inesperado: $SHA256"

PE_STATUS="FAIL"
if [[ "$PE_TYPE_RAW" == PE32* && "$PE_TYPE_RAW" == *"for MS Windows"* ]]; then
  PE_STATUS="PASS"
fi

VALIDATION_STATUS="FAIL"
if [[ "$PE_STATUS" == "PASS" ]]; then
  VALIDATION_STATUS="PASS"
fi

VALIDATED_AT_UTC="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" || fail "date não conseguiu gerar a data"
MACOS_VERSION="$(sw_vers -productVersion)" || fail "sw_vers não conseguiu identificar o macOS"
HOST_ARCH="$(uname -m)"
RELATIVE_EXECUTABLE="${EXECUTABLE_PATH#"$GAME_DIR"/}"

{
  printf '# Evidência de fixture — %s\n\n' "$(safe_text "$GAME_NAME")"
  printf -- '- Resultado da validação: **%s**\n' "$VALIDATION_STATUS"
  printf -- '- Versão informada pelo operador: `%s`\n' "$(safe_text "$VERSION_INPUT")"
  printf -- '- Origem informada pelo operador: `%s`\n' "$(safe_text "$ORIGIN_INPUT")"
  printf -- '- Data da validação (UTC): `%s`\n' "$VALIDATED_AT_UTC"
  printf -- '- Ambiente: macOS `%s`, arquitetura `%s`\n' "$MACOS_VERSION" "$HOST_ARCH"
  printf -- '- Pasta local: `<redacted>`\n'
  printf -- '- Pasta local disponível para leitura: **PASS**\n'
  printf -- '- Executável relativo selecionado: `%s`\n' "$(safe_text "$RELATIVE_EXECUTABLE")"
  printf -- '- Tipo identificado por `file`: `%s`\n' "$PE_TYPE"
  printf -- '- Tamanho: `%s` bytes\n' "$SIZE_BYTES"
  printf -- '- SHA-256: `%s`\n' "$SHA256"
  printf -- '- SHA-256 calculado: **PASS**\n'
  printf -- '- Arquivo com extensão `.exe`: **PASS**\n'
  printf -- '- Assinatura/tipo PE Windows: **%s**\n' "$PE_STATUS"
  printf -- '- Download feito por esta ferramenta: **NO**\n'
  printf -- '- Instalação, movimento ou remoção feita por esta ferramenta: **NONE**\n'
  printf '\n'
  printf '## Proveniência\n\n'
  printf 'A versão e a origem acima foram fornecidas no comando pelo operador; este script não tenta inferi-las nem acessa a internet.\n'
  printf 'O caminho absoluto da pasta local foi redigido para que esta evidência possa ser compartilhada com segurança.\n'
} > "$OUTPUT_PATH"

printf 'Evidência criada: %s\n' "$OUTPUT_PATH"
printf 'Executável: %s\n' "$RELATIVE_EXECUTABLE"
printf 'SHA-256: %s\n' "$SHA256"
printf 'Resultado: %s\n' "$VALIDATION_STATUS"

if [[ "$VALIDATION_STATUS" == "PASS" ]]; then
  exit 0
fi
exit 1
