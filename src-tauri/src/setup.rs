use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::error::CommandError;
use crate::launch::{self, LaunchRegistry};
use crate::runtimes::{backend_available, detect_runtimes, resolve_wine};
use crate::storage::{ensure_ponte_dir, expand_home, read_library, write_library};
use crate::types::{
    DxBackend, EngineGuess, Game, GameSetupPlan, PrefixStatus, Runtime, RuntimeKind, SetupItem,
    SetupItemStatus,
};

const DXVK_URL: &str =
    "https://github.com/Gcenx/DXVK-macOS/releases/download/v1.10.3-20230507-repack/dxvk-macOS-async-v1.10.3-20230507-repack.tar.gz";
const DXVK_MARKER: &str = ".ponte-dxvk-1.10.3";
const BREW_TIMEOUT: Duration = Duration::from_secs(20 * 60);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(180);

pub fn guess_engine(exe: &Path) -> EngineGuess {
    let Some(dir) = exe.parent() else {
        return EngineGuess::Unknown;
    };
    if dir.join("UnityPlayer.dll").is_file()
        || dir.join("GameAssembly.dll").is_file()
        || exe
            .file_stem()
            .map(|stem| dir.join(format!("{}_Data", stem.to_string_lossy())).is_dir())
            .unwrap_or(false)
    {
        return EngineGuess::Unity;
    }
    let name = exe
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let looks_shipping = name.contains("win64-shipping");
    let in_binaries_win64 = dir
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("win64"))
        && dir
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("binaries"));
    let mut ancestor = Some(dir);
    let mut has_engine = false;
    for _ in 0..5 {
        let Some(current) = ancestor else {
            break;
        };
        if current.join("Engine").is_dir() {
            has_engine = true;
            break;
        }
        ancestor = current.parent();
    }
    if looks_shipping || in_binaries_win64 || has_engine || dir.join("UnrealGame.exe").is_file()
    {
        return EngineGuess::Unreal;
    }
    EngineGuess::Unknown
}

fn brew_bin() -> Option<PathBuf> {
    [
        PathBuf::from("/opt/homebrew/bin/brew"),
        PathBuf::from("/usr/local/bin/brew"),
    ]
    .into_iter()
    .find(|path| path.is_file())
}

pub(crate) fn prefix_has_dxvk(prefix: &Path) -> bool {
    prefix.join("drive_c/windows/system32").join(DXVK_MARKER).is_file()
}

fn item(
    id: &str,
    label: &str,
    status: SetupItemStatus,
    detail: impl Into<String>,
) -> SetupItem {
    SetupItem {
        id: id.to_string(),
        label: label.to_string(),
        status,
        detail: detail.into(),
    }
}

pub fn recommended_stack_for_engine(engine: &EngineGuess) -> (RuntimeKind, DxBackend) {
    pick_stack(&detect_runtimes(), engine)
}

fn pick_stack(runtimes: &[Runtime], _engine: &EngineGuess) -> (RuntimeKind, DxBackend) {
    if runtimes
        .iter()
        .any(|item| item.kind == RuntimeKind::Gptk && item.found)
    {
        return (RuntimeKind::Gptk, DxBackend::D3dmetal);
    }
    if let Some(crossover) = runtimes
        .iter()
        .find(|item| item.kind == RuntimeKind::Crossover && item.found)
    {
        let path = Path::new(&crossover.wine64_path);
        if backend_available(path, &DxBackend::D3dmetal) {
            return (RuntimeKind::Crossover, DxBackend::D3dmetal);
        }
        return (RuntimeKind::Crossover, DxBackend::Dxvk);
    }
    if let Some(staging) = runtimes
        .iter()
        .find(|item| item.kind == RuntimeKind::GcenxStaging && item.found)
    {
        let path = Path::new(&staging.wine64_path);
        if backend_available(path, &DxBackend::Dxmt) {
            return (RuntimeKind::GcenxStaging, DxBackend::Dxmt);
        }
        return (RuntimeKind::GcenxStaging, DxBackend::Dxvk);
    }
    if runtimes
        .iter()
        .any(|item| item.kind == RuntimeKind::Winecx && item.found)
    {
        return (RuntimeKind::Winecx, DxBackend::Dxvk);
    }
    if runtimes
        .iter()
        .any(|item| item.kind == RuntimeKind::Winehq && item.found)
    {
        return (RuntimeKind::Winehq, DxBackend::Dxvk);
    }
    (RuntimeKind::GcenxStaging, DxBackend::Dxvk)
}

pub fn plan_for_game(game: &Game) -> GameSetupPlan {
    let exe = expand_home(&game.exe_path);
    let engine = guess_engine(&exe);
    let runtimes = detect_runtimes();
    let found = runtimes.iter().any(|item| item.found);
    let (recommended_runtime, recommended_backend) = pick_stack(&runtimes, &engine);
    let recommended = runtimes
        .iter()
        .find(|item| item.kind == recommended_runtime && item.found);
    let prefix = expand_home(&game.prefix_path);
    let prefix_ready = game.prefix_status == crate::types::PrefixStatus::Ready
        && prefix.join("system.reg").is_file();

    let wine_item = if let Some(runtime) = recommended {
        item(
            "runtime",
            "Wine",
            SetupItemStatus::Ready,
            format!("{} em {}", runtime.label, runtime.wine64_path),
        )
    } else if found {
        let any = runtimes.iter().find(|item| item.found).unwrap();
        item(
            "runtime",
            "Wine",
            SetupItemStatus::Ready,
            format!("{} (fallback)", any.label),
        )
    } else if brew_bin().is_some() {
        item(
            "runtime",
            "Wine",
            SetupItemStatus::Installable,
            "Nenhum wine64. O Ponte pode instalar o Wine Staging via Homebrew.",
        )
    } else {
        item(
            "runtime",
            "Wine",
            SetupItemStatus::Manual,
            "Instale o Homebrew e o Wine Staging, ou o Game Porting Toolkit.",
        )
    };

    let graphics_ready = match recommended_backend {
        DxBackend::D3dmetal => recommended
            .map(|runtime| backend_available(Path::new(&runtime.wine64_path), &DxBackend::D3dmetal))
            .unwrap_or(false),
        DxBackend::Dxmt => recommended
            .map(|runtime| backend_available(Path::new(&runtime.wine64_path), &DxBackend::Dxmt))
            .unwrap_or(false),
        DxBackend::Dxvk => {
            prefix_has_dxvk(&prefix)
                || recommended
                    .map(|runtime| {
                        backend_available(Path::new(&runtime.wine64_path), &DxBackend::Dxvk)
                    })
                    .unwrap_or(false)
        }
        DxBackend::Wined3d => found,
    };

    let graphics_item = if graphics_ready {
        item(
            "graphics",
            "DirectX",
            SetupItemStatus::Ready,
            match recommended_backend {
                DxBackend::D3dmetal => "D3DMetal disponível neste motor.",
                DxBackend::Dxmt => "DXMT disponível neste motor.",
                DxBackend::Dxvk => "DXVK-macOS pronto neste prefixo ou motor.",
                DxBackend::Wined3d => "WineD3D (limitado no Apple Silicon).",
            },
        )
    } else if recommended_backend == DxBackend::D3dmetal {
        item(
            "graphics",
            "DirectX",
            SetupItemStatus::Manual,
            "Este jogo pede D3DMetal. Instale o Game Porting Toolkit ou o CrossOver.",
        )
    } else {
        item(
            "graphics",
            "DirectX",
            SetupItemStatus::Installable,
            "O Ponte instala o DXVK-macOS (Gcenx) neste prefixo para o DirectX 11.",
        )
    };

    let prefix_item = if prefix_ready {
        item(
            "prefix",
            "Prefixo Wine",
            SetupItemStatus::Ready,
            prefix.display().to_string(),
        )
    } else if found || brew_bin().is_some() {
        item(
            "prefix",
            "Prefixo Wine",
            SetupItemStatus::Installable,
            "Ainda não criado. O Ponte corre wineboot neste jogo.",
        )
    } else {
        item(
            "prefix",
            "Prefixo Wine",
            SetupItemStatus::Missing,
            "Precisa de um wine64 antes de criar o prefixo.",
        )
    };

    let vcrun_ready = launch::vcrun_is_installed(&prefix);
    let vcrun_item = if vcrun_ready {
        item(
            "vcrun",
            "Visual C++",
            SetupItemStatus::Ready,
            "Microsoft Visual C++ 2015-2022 já está neste prefixo.",
        )
    } else if launch::vcrun_cache_ready() {
        item(
            "vcrun",
            "Visual C++",
            SetupItemStatus::Installable,
            "Já está na cache do Ponte; copia-se para este prefixo sem o instalador da Microsoft.",
        )
    } else if found || brew_bin().is_some() {
        item(
            "vcrun",
            "Visual C++",
            SetupItemStatus::Installable,
            "Muitos jogos pedem o runtime oficial da Microsoft neste prefixo.",
        )
    } else {
        item(
            "vcrun",
            "Visual C++",
            SetupItemStatus::Missing,
            "Instale o Wine primeiro.",
        )
    };

    let items = vec![wine_item, graphics_item, prefix_item, vcrun_item];
    let wine_blocked = items
        .iter()
        .any(|item| item.id == "runtime" && item.status == SetupItemStatus::Manual);
    let can_auto_fix = !wine_blocked
        && items
            .iter()
            .any(|item| item.status == SetupItemStatus::Installable);

    let summary = match engine {
        EngineGuess::Unity => {
            "Motor Unity: precisa de D3D11 (D3DMetal, DXMT ou DXVK), não WineD3D.".to_string()
        }
        EngineGuess::Unreal => {
            "Motor Unreal: o Ponte usa o exe Shipping e -dx11 (DXVK não faz D3D12).".to_string()
        }
        EngineGuess::Unknown => {
            "O Ponte vai garantir Wine, DirectX no prefixo, prefixo e Visual C++.".to_string()
        }
    };

    GameSetupPlan {
        game_id: game.id.clone(),
        engine_guess: engine,
        recommended_runtime,
        recommended_backend,
        items,
        can_auto_fix,
        summary,
    }
}

fn run_timed(
    mut child: std::process::Child,
    timeout: Duration,
    timeout_message: &str,
) -> Result<std::process::ExitStatus, CommandError> {
    let started = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                if started.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(CommandError::new("setup_failed", timeout_message));
                }
                std::thread::sleep(Duration::from_millis(250));
            }
            Err(err) => {
                return Err(CommandError::new(
                    "setup_failed",
                    format!("Falha ao consultar o processo: {err}"),
                ));
            }
        }
    }
}

fn brew_cask_installed(name: &str) -> bool {
    let Some(brew) = brew_bin() else {
        return false;
    };
    Command::new(brew)
        .args(["list", "--cask", name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn install_wine_staging(registry: &LaunchRegistry, game_id: &str) -> Result<(), CommandError> {
    if detect_runtimes().iter().any(|item| item.found) {
        launch::append_log(registry, game_id, "Wine já está instalado. Nada a descarregar.");
        return Ok(());
    }
    if brew_cask_installed("wine@staging") || brew_cask_installed("wine-stable") {
        launch::append_log(
            registry,
            game_id,
            "O cask do Wine já está no Homebrew. Nada a instalar.",
        );
        return Ok(());
    }
    let brew = brew_bin().ok_or_else(|| {
        CommandError::new(
            "setup_failed",
            "Homebrew não encontrado. Instale-o e tente de novo.",
        )
    })?;
    launch::append_log(
        registry,
        game_id,
        "A instalar Wine Staging (Gcenx) com Homebrew. Pode demorar vários minutos…",
    );
    let child = Command::new(&brew)
        .args(["install", "--cask", "--no-quarantine", "wine@staging"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            CommandError::new("setup_failed", format!("Falha ao iniciar o brew: {err}"))
        })?;
    let status = run_timed(
        child,
        BREW_TIMEOUT,
        "A instalação do Wine Staging excedeu 20 minutos.",
    )?;
    if !status.success() {
        launch::append_log(
            registry,
            game_id,
            "wine@staging falhou. A tentar wine-stable…",
        );
        let child = Command::new(&brew)
            .args(["install", "--cask", "--no-quarantine", "wine-stable"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| {
                CommandError::new("setup_failed", format!("Falha ao iniciar o brew: {err}"))
            })?;
        let status = run_timed(
            child,
            BREW_TIMEOUT,
            "A instalação do Wine Stable excedeu 20 minutos.",
        )?;
        if !status.success() {
            return Err(CommandError::new(
                "setup_failed",
                format!("O Homebrew não conseguiu instalar o Wine ({status})."),
            ));
        }
    }
    launch::append_log(registry, game_id, "Wine instalado.");
    Ok(())
}

fn find_named_dir(root: &Path, name: &str) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if dir.file_name().and_then(|value| value.to_str()) == Some(name) && dir.is_dir() {
            if dir.join("d3d11.dll").is_file() {
                return Some(dir);
            }
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                }
            }
        }
    }
    None
}

fn copy_dir_dlls(from: &Path, to: &Path) -> Result<(), CommandError> {
    fs::create_dir_all(to).map_err(|err| {
        CommandError::new("setup_failed", format!("Não foi possível criar {to:?}: {err}"))
    })?;
    for entry in fs::read_dir(from).map_err(|err| {
        CommandError::new("setup_failed", format!("Não foi possível ler DXVK: {err}"))
    })? {
        let entry = entry.map_err(|err| {
            CommandError::new("setup_failed", format!("Falha ao ler DXVK: {err}"))
        })?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("dll") {
            let dest = to.join(entry.file_name());
            fs::copy(&path, &dest).map_err(|err| {
                CommandError::new(
                    "setup_failed",
                    format!("Falha ao copiar {}: {err}", path.display()),
                )
            })?;
        }
    }
    Ok(())
}

fn ensure_dxvk(
    registry: &LaunchRegistry,
    game_id: &str,
    wine64: &str,
    prefix: &Path,
) -> Result<(), CommandError> {
    if prefix_has_dxvk(prefix) {
        launch::append_log(registry, game_id, "DXVK-macOS já está neste prefixo.");
        return Ok(());
    }
    let cache = ensure_ponte_dir()?.join("cache");
    fs::create_dir_all(&cache).map_err(|err| {
        CommandError::new("setup_failed", format!("Não foi possível criar a cache: {err}"))
    })?;
    let archive = cache.join("dxvk-macos-1.10.3.tar.gz");
    if !archive.is_file() || archive.metadata().map(|meta| meta.len()).unwrap_or(0) < 100_000 {
        launch::append_log(
            registry,
            game_id,
            "A descarregar DXVK-macOS 1.10.3 (Gcenx)…",
        );
        let dest = archive.to_string_lossy().into_owned();
        let child = Command::new("curl")
            .args(["-fsSL", "--retry", "3", "-o", &dest, DXVK_URL])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| {
                CommandError::new("setup_failed", format!("Falha ao descarregar o DXVK: {err}"))
            })?;
        let status = run_timed(
            child,
            DOWNLOAD_TIMEOUT,
            "O download do DXVK excedeu 3 minutos.",
        )?;
        if !status.success() || !archive.is_file() {
            return Err(CommandError::new(
                "setup_failed",
                "Não foi possível descarregar o DXVK-macOS.",
            ));
        }
    }
    let extract = cache.join("dxvk-macos-1.10.3");
    let cached_x64 = find_named_dir(&extract, "x64");
    let from_cache = cached_x64.is_some();
    if from_cache {
        launch::append_log(
            registry,
            game_id,
            "A copiar DXVK-macOS da cache. Nada a descarregar.",
        );
    }
    let x64 = if let Some(existing) = cached_x64 {
        existing
    } else {
        fs::create_dir_all(&extract).ok();
        let status = Command::new("tar")
            .args(["-xzf"])
            .arg(&archive)
            .arg("-C")
            .arg(&extract)
            .status()
            .map_err(|err| {
                CommandError::new("setup_failed", format!("Falha ao extrair DXVK: {err}"))
            })?;
        if !status.success() {
            return Err(CommandError::new(
                "setup_failed",
                "Não foi possível extrair o DXVK-macOS.",
            ));
        }
        find_named_dir(&extract, "x64").ok_or_else(|| {
            CommandError::new("setup_failed", "O arquivo DXVK não contém a pasta x64.")
        })?
    };
    let system32 = prefix.join("drive_c/windows/system32");
    copy_dir_dlls(&x64, &system32)?;
    if let Some(x32) = find_named_dir(&extract, "x32") {
        let _ = copy_dir_dlls(&x32, &prefix.join("drive_c/windows/syswow64"));
    }
    fs::write(system32.join(DXVK_MARKER), b"gcenx-dxvk-macos\n").ok();
    for dll in ["d3d11", "d3d10core", "dxgi"] {
        let _ = Command::new(wine64)
            .env("WINEPREFIX", prefix)
            .args([
                "reg",
                "add",
                r"HKCU\Software\Wine\DllOverrides",
                "/v",
                dll,
                "/d",
                "native,builtin",
                "/f",
            ])
            .status();
    }
    launch::append_log(
        registry,
        game_id,
        if from_cache {
            "DXVK-macOS copiado da cache para este prefixo."
        } else {
            "DXVK-macOS instalado neste prefixo (DirectX 11 → Vulkan/Metal)."
        },
    );
    Ok(())
}

fn apply_stack(game: &mut Game, runtime: RuntimeKind, backend: DxBackend) {
    game.runtime_kind = runtime;
    game.dx_backend = backend;
}

pub(crate) fn find_unreal_shipping(exe: &Path) -> Option<PathBuf> {
    let name = exe
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if name.contains("win64-shipping") {
        return None;
    }
    let parent = exe.parent()?;
    let search_roots = [parent, parent.parent().unwrap_or(parent)];
    for root in search_roots {
        if let Some(found) = shipping_in_tree(root) {
            if found != exe {
                return Some(found);
            }
        }
    }
    None
}

fn first_shipping_in(dir: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    let mut matches: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| {
                        let lower = name.to_ascii_lowercase();
                        lower.ends_with(".exe") && lower.contains("win64-shipping")
                    })
                    .unwrap_or(false)
        })
        .collect();
    matches.sort();
    matches.into_iter().next()
}

fn shipping_in_tree(root: &Path) -> Option<PathBuf> {
    if let Some(found) = first_shipping_in(&root.join("Binaries/Win64")) {
        return Some(found);
    }
    let entries = fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = first_shipping_in(&path.join("Binaries/Win64")) {
                return Some(found);
            }
        }
    }
    None
}

pub(crate) fn with_dx11_arg(args: &str) -> String {
    let has_graphics = args.split_whitespace().any(|arg| {
        matches!(
            arg.to_ascii_lowercase().as_str(),
            "-dx11" | "-dx12" | "-d3d11" | "-d3d12" | "-vulkan" | "-opengl"
        )
    });
    if has_graphics {
        return args.to_string();
    }
    let trimmed = args.trim();
    if trimmed.is_empty() {
        "-dx11".to_string()
    } else {
        format!("{trimmed} -dx11")
    }
}

fn apply_unreal_launch(game: &mut Game, engine: &EngineGuess, backend: DxBackend) {
    if *engine != EngineGuess::Unreal {
        return;
    }
    let exe = expand_home(&game.exe_path);
    if let Some(shipping) = find_unreal_shipping(&exe) {
        game.exe_path = shipping.to_string_lossy().into_owned();
    }
    if matches!(
        backend,
        DxBackend::Dxvk | DxBackend::Dxmt | DxBackend::Wined3d
    ) {
        game.args = with_dx11_arg(&game.args);
    }
}

pub fn game_is_ready(game: &Game) -> bool {
    if resolve_wine(&game.runtime_kind).is_none() {
        return false;
    }
    let prefix = expand_home(&game.prefix_path);
    if game.prefix_status != PrefixStatus::Ready || !prefix.join("system.reg").is_file() {
        return false;
    }
    if !launch::vcrun_is_installed(&prefix) {
        return false;
    }
    match game.dx_backend {
        DxBackend::Dxvk => {
            prefix_has_dxvk(&prefix)
                || resolve_wine(&game.runtime_kind)
                    .map(|runtime| {
                        backend_available(Path::new(&runtime.wine64_path), &DxBackend::Dxvk)
                    })
                    .unwrap_or(false)
        }
        DxBackend::D3dmetal => resolve_wine(&game.runtime_kind)
            .map(|runtime| {
                backend_available(Path::new(&runtime.wine64_path), &DxBackend::D3dmetal)
            })
            .unwrap_or(false),
        DxBackend::Dxmt => resolve_wine(&game.runtime_kind)
            .map(|runtime| backend_available(Path::new(&runtime.wine64_path), &DxBackend::Dxmt))
            .unwrap_or(false),
        DxBackend::Wined3d => true,
    }
}

pub fn ensure_prefix_deps(registry: &LaunchRegistry, game_id: &str) -> Result<Game, CommandError> {
    let library = read_library()?;
    let snapshot = library
        .games
        .iter()
        .find(|item| item.id == game_id)
        .cloned()
        .ok_or_else(|| CommandError::new("not_found", "Jogo não encontrado."))?;

    if snapshot.prefix_status != PrefixStatus::Ready {
        launch::append_log(registry, game_id, "A criar o prefixo Wine…");
        launch::create_prefix(registry, &snapshot, false)?;
        let mut library = read_library()?;
        if let Some(stored) = library.games.iter_mut().find(|item| item.id == game_id) {
            stored.prefix_status = PrefixStatus::Ready;
            stored.prefix_error = None;
            write_library(&library)?;
        }
    }

    let library = read_library()?;
    let game = library
        .games
        .iter()
        .find(|item| item.id == game_id)
        .cloned()
        .ok_or_else(|| CommandError::new("not_found", "Jogo não encontrado."))?;

    if game.dx_backend == DxBackend::Dxvk {
        if let Some(runtime) = resolve_wine(&game.runtime_kind) {
            ensure_dxvk(
                registry,
                game_id,
                &runtime.wine64_path,
                &expand_home(&game.prefix_path),
            )?;
        }
    }

    launch::install_vcrun(registry, &game)?;
    let library = read_library()?;
    library
        .games
        .into_iter()
        .find(|item| item.id == game_id)
        .ok_or_else(|| CommandError::new("not_found", "Jogo não encontrado."))
}

pub fn prepare_game(registry: &LaunchRegistry, game_id: &str) -> Result<Game, CommandError> {
    let mut library = read_library()?;
    let game = library
        .games
        .iter()
        .find(|item| item.id == game_id)
        .cloned()
        .ok_or_else(|| CommandError::new("not_found", "Jogo não encontrado."))?;

    launch::set_state(registry, game_id, crate::types::LaunchState::Starting);
    launch::append_log(registry, game_id, "A analisar o que este jogo precisa…");
    let plan = plan_for_game(&game);
    launch::append_log(registry, game_id, &plan.summary);

    if !detect_runtimes().iter().any(|item| item.found) {
        install_wine_staging(registry, game_id)?;
    }

    let runtimes = detect_runtimes();
    let (runtime_kind, backend) = pick_stack(&runtimes, &plan.engine_guess);
    if resolve_wine(&runtime_kind).is_none() {
        launch::set_state(registry, game_id, crate::types::LaunchState::Error);
        return Err(CommandError::new(
            "runtime_missing",
            "Ainda não há wine64. Instale o Wine Staging ou o GPTK e tente de novo.",
        ));
    }

    {
        let stored = library
            .games
            .iter_mut()
            .find(|item| item.id == game_id)
            .ok_or_else(|| CommandError::new("not_found", "Jogo não encontrado."))?;
        apply_stack(stored, runtime_kind, backend);
        apply_unreal_launch(stored, &plan.engine_guess, backend);
        if stored.exe_path != game.exe_path {
            launch::append_log(
                registry,
                game_id,
                &format!("A usar o executável Unreal Shipping: {}", stored.exe_path),
            );
        }
        if stored.args != game.args {
            launch::append_log(
                registry,
                game_id,
                &format!("Argumentos: {}", stored.args),
            );
        }
    };
    write_library(&library)?;
    ensure_prefix_deps(registry, game_id)?;

    launch::append_log(
        registry,
        game_id,
        &format!(
            "Preparação concluída. Motor {} + {:?}. Pode iniciar.",
            match runtime_kind {
                RuntimeKind::Gptk => "Game Porting Toolkit",
                RuntimeKind::GcenxStaging => "Wine Staging",
                RuntimeKind::Winehq => "WineHQ Stable",
                RuntimeKind::Winecx => "Wine CX",
                RuntimeKind::Crossover => "CrossOver",
                RuntimeKind::Custom => "Wine manual",
            },
            backend
        ),
    );
    launch::set_state(registry, game_id, crate::types::LaunchState::Idle);
    let library = read_library()?;
    library
        .games
        .into_iter()
        .find(|item| item.id == game_id)
        .ok_or_else(|| CommandError::new("not_found", "Jogo não encontrado."))
}

pub fn install_runtime(kind: &RuntimeKind) -> Result<(), CommandError> {
    match kind {
        RuntimeKind::GcenxStaging | RuntimeKind::Winehq => {
            let dummy = LaunchRegistry::new();
            install_wine_staging(&dummy, "setup")?;
            Ok(())
        }
        RuntimeKind::Gptk => Err(CommandError::new(
            "setup_failed",
            "O Game Porting Toolkit não se instala daqui. Use o site da Apple ou o tap Homebrew apple/apple.",
        )),
        RuntimeKind::Crossover => Err(CommandError::new(
            "setup_failed",
            "O CrossOver é pago. Instale-o em codeweavers.com e o Ponte detecta-o.",
        )),
        RuntimeKind::Winecx | RuntimeKind::Custom => Err(CommandError::new(
            "setup_failed",
            "Este motor não tem instalador automático. Aponte o binário em Ajustes.",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{find_unreal_shipping, guess_engine, with_dx11_arg};
    use crate::types::EngineGuess;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ponte-setup-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn guesses_unity_from_data_folder() {
        let dir = temp_dir("unity");
        fs::create_dir_all(dir.join("REPO_Data")).unwrap();
        let exe = dir.join("REPO.exe");
        fs::write(&exe, b"").unwrap();
        assert_eq!(guess_engine(&exe), EngineGuess::Unity);
    }

    #[test]
    fn guesses_unreal_from_engine_folder() {
        let dir = temp_dir("unreal");
        fs::create_dir_all(dir.join("Engine")).unwrap();
        let exe = dir.join("ShooterGame.exe");
        fs::write(&exe, b"").unwrap();
        assert_eq!(guess_engine(&exe), EngineGuess::Unreal);
    }

    #[test]
    fn unknown_without_markers() {
        let dir = temp_dir("plain");
        let exe = dir.join("notepad.exe");
        fs::write(&exe, b"").unwrap();
        assert_eq!(guess_engine(&exe), EngineGuess::Unknown);
    }

    #[test]
    fn finds_unreal_shipping_next_to_bootstrap() {
        let dir = temp_dir("shipping");
        fs::create_dir_all(dir.join("Chameleon/Binaries/Win64")).unwrap();
        let bootstrap = dir.join("PenguinHotel.exe");
        let shipping = dir.join("Chameleon/Binaries/Win64/PenguinHotel-Win64-Shipping.exe");
        fs::write(&bootstrap, b"").unwrap();
        fs::write(&shipping, b"").unwrap();
        assert_eq!(find_unreal_shipping(&bootstrap), Some(shipping.clone()));
        assert_eq!(guess_engine(&shipping), EngineGuess::Unreal);
    }

    #[test]
    fn adds_dx11_unless_user_chose_another_rhi() {
        assert_eq!(with_dx11_arg(""), "-dx11");
        assert_eq!(with_dx11_arg("-windowed"), "-windowed -dx11");
        assert_eq!(with_dx11_arg("-dx12"), "-dx12");
    }
}
