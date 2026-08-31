# Ponte D3D11 probe

`d3d11-probe.exe` é um executável Windows x64 pequeno e autocontido para
validar a parte mínima da pilha D3D11 usada pela Ponte:

1. registra uma classe Win32 e cria uma janela de 1×1 que permanece oculta;
2. chama `D3D11CreateDeviceAndSwapChain`;
3. obtém o adaptador DXGI e seus identificadores;
4. cria um render target, limpa um frame e chama `Present`;
5. imprime exatamente uma linha JSON em stdout.

O executável não usa DLLs de terceiros. As únicas bibliotecas vinculadas são
`d3d11.lib`, `dxgi.lib` e `user32.lib`, fornecidas pelo Windows SDK e
carregadas pelo próprio Windows. O runtime C é ligado estaticamente com
`/MT`.

## Uso

No Windows, a execução padrão usa o adaptador de hardware:

```powershell
.\d3d11-probe.exe
```

Para uma validação determinística em CI, o probe aceita `--warp`, usando o
Microsoft WARP (renderização por software) para exercitar a mesma criação de
janela, device, swap chain, clear e `Present`:

```powershell
.\d3d11-probe.exe --warp
```

O processo retorna `0` quando o device é criado, um frame é limpo e `Present`
passa. Retorna `1` para falha do probe e `2` para argumento desconhecido.

## Formato JSON

Todos os caminhos imprimem uma única linha e usam os mesmos campos:

```json
{"ok":true,"featureLevel":"11_0","adapter":"Microsoft Basic Render Driver","vendorId":5140,"deviceId":140,"hresult":null,"stage":null}
```

Em uma falha, os campos já conhecidos são preservados e `hresult`/`stage`
identificam a operação que falhou:

```json
{"ok":false,"featureLevel":"11_0","adapter":"...","vendorId":5140,"deviceId":140,"hresult":"0x887A0001","stage":"present"}
```

Os valores `featureLevel`, `adapter`, `vendorId` e `deviceId` são `null` quando
a falha ocorre antes de a informação correspondente estar disponível. O
`hresult` é formatado como hexadecimal de 32 bits. O `adapter` é convertido
de UTF-16 para UTF-8 e escapado como JSON.

## Compilação manual

Abra o **x64 Native Tools Command Prompt for VS 2022** com um Windows SDK
instalado e execute a partir da raiz do repositório:

```bat
if not exist build mkdir build
cl.exe /nologo /std:c++17 /W4 /WX /EHsc /MT /O2 /Brepro ^
  /DWIN32_LEAN_AND_MEAN /DNOMINMAX /DUNICODE /D_UNICODE ^
  /Fe:build\d3d11-probe.exe /Fo:build\d3d11-probe.obj ^
  tools\d3d11-probe\d3d11-probe.cpp ^
  /link /SUBSYSTEM:CONSOLE /MACHINE:X64 /Brepro /INCREMENTAL:NO /OPT:REF /OPT:ICF ^
  d3d11.lib dxgi.lib user32.lib
```

As flags relevantes para reprodutibilidade são `/MT`, `/O2`, `/Brepro` e
`/SUBSYSTEM:CONSOLE`. Para comparar builds, use a mesma versão do Visual
Studio, do MSVC e do Windows SDK. O workflow
`.github/workflows/d3d11-probe.yml` executa essa compilação em `windows-2022`.

## SHA-256: gerar e verificar

Depois de compilar, gere um manifesto no PowerShell:

```powershell
$hash = (Get-FileHash -Algorithm SHA256 .\build\d3d11-probe.exe).Hash.ToLowerInvariant()
"$hash  d3d11-probe.exe" | Out-File .\build\d3d11-probe.exe.sha256 -Encoding ascii
Get-Content .\build\d3d11-probe.exe.sha256
```

Verifique posteriormente comparando o hash calculado com o manifesto:

```powershell
$expected = ((Get-Content .\build\d3d11-probe.exe.sha256) -split '\s+')[0].ToLowerInvariant()
$actual = (Get-FileHash -Algorithm SHA256 .\build\d3d11-probe.exe).Hash.ToLowerInvariant()
if ($actual -ne $expected) { throw "SHA-256 mismatch" }
Write-Host "SHA-256 OK: $actual"
```

Também é possível usar o utilitário nativo do Windows:

```bat
certutil -hashfile build\d3d11-probe.exe SHA256
```

O workflow publica `d3d11-probe.exe` e seu manifesto como o artifact
`d3d11-probe-windows-x64`. O job valida metadados PE x64, subsistema console,
ausência de importação do CRT dinâmico, JSON do probe WARP e, quando
disponível, também executa o caminho de hardware.

## Limitações conhecidas

- O probe é construído somente para Windows x64; não é um binário macOS.
- O caminho padrão testa um adaptador de hardware, mas a disponibilidade do
  adaptador depende da máquina. `--warp` prova o contrato D3D11/Win32/Present,
  porém não prova uma GPU física ou a tradução DXVK no Wine.
- Apenas feature levels 11.1 e 11.0 são solicitados. O fallback para 11.0
  cobre sistemas que rejeitam a enumeração de 11.1.
- A janela é oculta e mede 1×1; o probe não valida fullscreen, input,
  shaders específicos do jogo, D3D12, Vulkan, áudio ou anti-cheat.
- A compilação do workflow garante um executável válido e uma execução WARP;
  a validação de DXVK/Wine no macOS ainda precisa executar este binário dentro
  de um prefixo Wine real e registrar o resultado no app.
