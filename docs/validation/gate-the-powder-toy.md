# Gate externo — The Powder Toy

Este documento descreve a validação externa do **The Powder Toy** para o
Ponte. O gate prova apenas que uma cópia local já existente contém um
executável Windows PE identificável e que o SHA-256 dessa cópia foi registrado
em uma evidência nova. Ele não prova que o jogo abre no Wine, que a renderização
funciona ou que a cópia veio de uma origem legítima.

## Limite de segurança

`scripts/validate-game-gate.sh` é deliberadamente um validador local. Ele:

- lê a pasta indicada e seleciona um `.exe` regular dentro dela;
- usa `file` para exigir uma identificação `PE32`/`PE32+` cujo texto contenha
  `for MS Windows`;
- calcula o SHA-256 e registra tamanho, tipo, executável relativo e ambiente;
- cria somente o arquivo de evidência indicado, se ele ainda não existir.

O script não baixa, instala, executa, move, copia, renomeia ou apaga o jogo; não
cria a pasta de saída; e não acessa a internet. A pasta de saída deve existir,
ficar fora da pasta do jogo e apontar para um caminho novo. A versão e a origem
são metadados fornecidos pelo operador e aparecem como tal na evidência.

## Pré-condições

1. macOS com `find`, `file`, `shasum`, `stat` e `sw_vers` disponíveis.
2. Uma pasta local do The Powder Toy já obtida pelo operador.
3. Uma pasta de evidências já existente, fora da pasta do jogo.
4. A versão e a URL de origem confirmadas pelo operador antes da execução.

O validador não tenta descobrir a versão, verificar a URL ou comparar o hash
com um catálogo remoto. Para evitar validar o executável errado, use
`--executable` quando a pasta contiver mais de um `.exe`.

## Execução

A partir da raiz do repositório:

```bash
scripts/validate-game-gate.sh \
  "/caminho/local/The Powder Toy" \
  --game-name "The Powder Toy" \
  --version "<versão confirmada>" \
  --origin "https://powdertoy.co.uk/" \
  --executable "/caminho/local/The Powder Toy/<arquivo>.exe" \
  --output "/caminho/existente/evidence/the-powder-toy-<data>.md"
```

O `--executable` é opcional somente quando existe exatamente um arquivo `.exe`
regular dentro da pasta, incluindo subpastas. Todos os caminhos precisam ser
absolutos. O arquivo de saída não pode existir, mesmo que seja um link
simbólico quebrado; isso evita sobrescrever evidência ou seguir um link
inesperado.

## Resultado e evidência

O comando retorna:

| Código | Significado |
| --- | --- |
| `0` | `.exe` encontrado e reconhecido como PE Windows; evidência criada. |
| `1` | `.exe` encontrado, mas a identificação PE falhou; evidência criada com `FAIL`. |
| `2` | Uso inválido, ferramenta ausente, pasta/saída inválida ou pré-condição ausente. |

Uma evidência válida deve conter, no mínimo:

- resultado `PASS` ou `FAIL`;
- versão e origem informadas pelo operador;
- executável relativo selecionado;
- tipo retornado por `file`;
- tamanho em bytes e SHA-256 de 64 dígitos hexadecimais;
- confirmação do tipo PE e das operações proibidas não realizadas.

O caminho absoluto da pasta do jogo é redigido como `<redacted>` para permitir
compartilhar a evidência sem expor a máquina. O hash identifica os bytes do
executável selecionado, não a pasta inteira.

## Critério do gate

Considere o gate externo **PASS** somente quando:

1. o comando retornar `0`;
2. a evidência nova for revisada e associada à versão/origem corretas;
3. o SHA-256 for preservado para comparação posterior;
4. qualquer teste de execução no Ponte/Wine for registrado separadamente, sem
   confundir compatibilidade local com esta validação estática.

Se o The Powder Toy não estiver disponível localmente, o gate permanece
`NOT RUN`. Não baixe o jogo por este script para tentar completar a validação.
