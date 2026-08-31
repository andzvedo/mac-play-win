use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use crate::attempts;
use crate::discover::is_helper_exe;
use crate::env_backend::env_for_backend;
use crate::error::CommandError;
use crate::paths::{is_windows_exe_path, prefix_is_owned};
use crate::runtimes::resolve_wine;
use crate::storage::{ensure_ponte_dir, expand_home, read_settings};
use crate::types::{
    DxBackend, Game, GameLog, LaunchAttempt, LaunchAttemptStatus, LaunchFailure, LaunchState,
};

const LOG_CAP: usize = 500;
const WINEBOOT_TIMEOUT: Duration = Duration::from_secs(180);
const VCRUN_TIMEOUT: Duration = Duration::from_secs(300);
const VCRUN_URL: &str = "https://aka.ms/vs/17/release/vc_redist.x64.exe";
const VCRUN_DLLS: &[&str] = &[
    "vcruntime140",
    "vcruntime140_1",
    "msvcp140",
    "msvcp140_1",
    "msvcp140_2",
    "concrt140",
    "vccorlib140",
];
const VCRUN_NATIVE_FILES: &[&str] = &["vcruntime140.dll", "vcruntime140_1.dll", "msvcp140.dll"];
const VCRUN_CACHE_FILES: &[&str] = &[
    "vcruntime140.dll",
    "vcruntime140_1.dll",
    "msvcp140.dll",
    "msvcp140_1.dll",
    "msvcp140_2.dll",
    "concrt140.dll",
    "vccorlib140.dll",
];
const VCRUN_MARKER: &str = ".ponte-vcrun-2015-x64";
const WINE_BUILTIN_MARK: &[u8] = b"Wine builtin DLL";

pub struct LaunchRegistry {
    pub processes: Mutex<HashMap<String, Child>>,
    pub logs: Mutex<HashMap<String, Vec<String>>>,
    pub states: Mutex<HashMap<String, LaunchState>>,
    pub active_attempts: Mutex<HashMap<String, String>>,
}

impl LaunchRegistry {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
            logs: Mutex::new(HashMap::new()),
            states: Mutex::new(HashMap::new()),
            active_attempts: Mutex::new(HashMap::new()),
        }
    }
}

fn now_stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() % 86400)
        .unwrap_or(0);
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

pub fn append_log(registry: &LaunchRegistry, game_id: &str, message: &str) {
    let line = format!("[{}] {message}", now_stamp());
    {
        let mut logs = registry.logs.lock().expect("logs mutex");
        let lines = logs.entry(game_id.to_string()).or_default();
        lines.push(line.clone());
        if lines.len() > LOG_CAP {
            let extra = lines.len() - LOG_CAP;
            lines.drain(0..extra);
        }
    }
    let attempt_id = registry
        .active_attempts
        .lock()
        .expect("attempt mutex")
        .get(game_id)
        .cloned();
    if let Some(attempt_id) = attempt_id {
        if let Ok(attempt) = attempts::read(game_id, &attempt_id) {
            let _ = attempts::append_log(&attempt, &line);
        }
    }
}

fn active_attempt(registry: &LaunchRegistry, game_id: &str) -> Option<String> {
    registry
        .active_attempts
        .lock()
        .expect("attempt mutex")
        .get(game_id)
        .cloned()
}

fn finish_attempt(
    registry: &LaunchRegistry,
    game_id: &str,
    status: LaunchAttemptStatus,
    exit_code: Option<i32>,
    failure: Option<LaunchFailure>,
) {
    let Some(attempt_id) = active_attempt(registry, game_id) else {
        return;
    };
    if let Ok(mut attempt) = attempts::read(game_id, &attempt_id) {
        attempt.status = status;
        attempt.updated_at = now_epoch();
        attempt.finished_at = Some(now_epoch());
        attempt.exit_code = exit_code;
        attempt.duration_ms = attempt.started_at.parse::<u64>().ok().and_then(|started| {
            now_epoch()
                .parse::<u64>()
                .ok()
                .map(|now| now.saturating_sub(started) * 1000)
        });
        attempt.failure = failure;
        let _ = attempts::save(&attempt);
    }
    registry
        .active_attempts
        .lock()
        .expect("attempt mutex")
        .remove(game_id);
}

fn now_epoch() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

pub fn set_state(registry: &LaunchRegistry, game_id: &str, state: LaunchState) {
    registry
        .states
        .lock()
        .expect("state mutex")
        .insert(game_id.to_string(), state);
}

pub fn current_state(registry: &LaunchRegistry, game_id: &str) -> LaunchState {
    reap(registry, game_id);
    registry
        .states
        .lock()
        .expect("state mutex")
        .get(game_id)
        .cloned()
        .unwrap_or(LaunchState::Idle)
}

fn reap(registry: &LaunchRegistry, game_id: &str) {
    let mut processes = registry.processes.lock().expect("process mutex");
    if let Some(child) = processes.get_mut(game_id) {
        match child.try_wait() {
            Ok(Some(status)) => {
                processes.remove(game_id);
                if status.success() {
                    set_state(registry, game_id, LaunchState::Idle);
                    append_log(registry, game_id, "Processo terminou.");
                    finish_attempt(
                        registry,
                        game_id,
                        LaunchAttemptStatus::Succeeded,
                        status.code(),
                        None,
                    );
                } else {
                    set_state(registry, game_id, LaunchState::Error);
                    append_log(
                        registry,
                        game_id,
                        &format!("Processo saiu com código {status}."),
                    );
                    hint_graphics_failure(registry, game_id);
                    finish_attempt(
                        registry,
                        game_id,
                        LaunchAttemptStatus::Failed,
                        status.code(),
                        Some(LaunchFailure {
                            code: "process_exit".to_string(),
                            message: format!("Processo saiu com código {status}."),
                            retryable: true,
                        }),
                    );
                }
            }
            Ok(None) => {}
            Err(err) => {
                processes.remove(game_id);
                set_state(registry, game_id, LaunchState::Error);
                append_log(
                    registry,
                    game_id,
                    &format!("Falha ao consultar processo: {err}"),
                );
                finish_attempt(
                    registry,
                    game_id,
                    LaunchAttemptStatus::Failed,
                    None,
                    Some(LaunchFailure {
                        code: "process_wait".to_string(),
                        message: err.to_string(),
                        retryable: true,
                    }),
                );
            }
        }
    }
}

fn backend_label(backend: &DxBackend) -> &'static str {
    match backend {
        DxBackend::D3dmetal => "D3DMetal",
        DxBackend::Dxmt => "DXMT",
        DxBackend::Dxvk => "DXVK-macOS",
        DxBackend::Wined3d => "WineD3D",
    }
}

fn hint_graphics_failure(registry: &LaunchRegistry, game_id: &str) {
    const NEEDLE: &str = "wined3d_select_feature_level";
    const HINT: &str = "O WineD3D do WineHQ Stable não consegue o DirectX 11 neste Mac. Instale o Wine Staging (Gcenx) ou o Game Porting Toolkit e mude o motor do jogo.";
    let should_hint = {
        let logs = registry.logs.lock().expect("logs mutex");
        match logs.get(game_id) {
            Some(lines) => {
                lines.iter().any(|line| line.contains(NEEDLE))
                    && !lines
                        .iter()
                        .any(|line| line.contains("não consegue o DirectX 11"))
            }
            None => false,
        }
    };
    if should_hint {
        append_log(registry, game_id, HINT);
    }
}

fn pipe_output(registry: Arc<LaunchRegistry>, game_id: String, child: &mut Child) {
    if let Some(stdout) = child.stdout.take() {
        let logs_registry = Arc::clone(&registry);
        let id = game_id.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                append_log(&logs_registry, &id, &line);
            }
        });
    }
    if let Some(stderr) = child.stderr.take() {
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                append_log(&registry, &game_id, &line);
            }
        });
    }
}

pub fn create_prefix(
    registry: &LaunchRegistry,
    game: &Game,
    wipe: bool,
) -> Result<(), CommandError> {
    let runtime = resolve_wine(&game.runtime_kind).ok_or_else(|| {
        CommandError::new(
            "runtime_missing",
            "Nenhum wine64 encontrado para criar o prefixo.",
        )
    })?;
    let prefix = expand_home(&game.prefix_path);
    if wipe {
        let settings = read_settings()?;
        let prefixes_dir = expand_home(&settings.prefixes_dir);
        if !prefix_is_owned(&prefix, &prefixes_dir, &game.id) {
            return Err(CommandError::new(
                "prefix_failed",
                "O prefixo está fora da pasta do Ponte; recusar apagar.",
            ));
        }
        if prefix.exists() {
            append_log(registry, &game.id, "A apagar o prefixo anterior.");
            std::fs::remove_dir_all(&prefix).map_err(|err| {
                CommandError::new(
                    "prefix_failed",
                    format!("Não foi possível apagar o prefixo: {err}"),
                )
            })?;
        }
    }
    std::fs::create_dir_all(&prefix).ok();
    run_wineboot(registry, &game.id, &runtime.wine64_path, &prefix)?;
    append_log(registry, &game.id, "Prefixo pronto.");
    Ok(())
}

fn run_wineboot(
    registry: &LaunchRegistry,
    game_id: &str,
    wine64: &str,
    prefix: &std::path::Path,
) -> Result<(), CommandError> {
    append_log(
        registry,
        game_id,
        &format!("WINEPREFIX={} wineboot --init", prefix.display()),
    );
    let mut child = Command::new(wine64)
        .env("WINEPREFIX", prefix)
        .args(["wineboot", "--init"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| CommandError::new("prefix_failed", format!("wineboot falhou: {err}")))?;

    let stderr_pipe = child.stderr.take();
    let stderr_thread = std::thread::spawn(move || {
        let mut buf = String::new();
        if let Some(mut pipe) = stderr_pipe {
            let _ = pipe.read_to_string(&mut buf);
        }
        buf
    });

    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if started.elapsed() >= WINEBOOT_TIMEOUT {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = stderr_thread.join();
                    return Err(CommandError::new(
                        "prefix_failed",
                        "wineboot excedeu 3 minutos e foi interrompido.",
                    ));
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(err) => {
                return Err(CommandError::new(
                    "prefix_failed",
                    format!("Falha ao consultar wineboot: {err}"),
                ));
            }
        }
    };

    let stderr = stderr_thread.join().unwrap_or_default();
    if !stderr.trim().is_empty() {
        append_log(registry, game_id, stderr.trim());
    }
    if !status.success() {
        return Err(CommandError::new(
            "prefix_failed",
            format!("wineboot saiu com {status}"),
        ));
    }
    Ok(())
}

fn vcrun_installer_ok(status: &std::process::ExitStatus) -> bool {
    match status.code() {
        Some(0) | Some(3010) => true,
        _ => status.success(),
    }
}

fn dll_is_wine_builtin(path: &Path) -> bool {
    let Ok(data) = fs::read(path) else {
        return true;
    };
    data.windows(WINE_BUILTIN_MARK.len())
        .any(|window| window == WINE_BUILTIN_MARK)
}

fn system32_dir(prefix: &Path) -> PathBuf {
    prefix.join("drive_c/windows/system32")
}

fn vcrun_files_present(dir: &Path) -> bool {
    VCRUN_NATIVE_FILES.iter().all(|name| {
        let path = dir.join(name);
        path.is_file() && !dll_is_wine_builtin(&path)
    })
}

pub(crate) fn vcrun_is_installed(prefix: &Path) -> bool {
    vcrun_files_present(&system32_dir(prefix))
}

pub(crate) fn vcrun_cache_ready() -> bool {
    vcrun_cache_dir()
        .map(|dir| vcrun_files_present(&dir))
        .unwrap_or(false)
}

fn vcrun_cache_dir() -> Result<PathBuf, CommandError> {
    let dir = ensure_ponte_dir()?.join("cache/vcrun2015-x64");
    fs::create_dir_all(&dir).map_err(|err| {
        CommandError::new(
            "prefix_failed",
            format!("Não foi possível criar a cache do Visual C++: {err}"),
        )
    })?;
    Ok(dir)
}

fn copy_vcrun_dlls(from: &Path, to: &Path) -> bool {
    let _ = fs::create_dir_all(to);
    for name in VCRUN_CACHE_FILES {
        let src = from.join(name);
        if src.is_file() && !dll_is_wine_builtin(&src) {
            let _ = fs::copy(&src, to.join(name));
        }
    }
    vcrun_files_present(to)
}

fn capture_vcrun_to_cache(prefix: &Path) {
    let Ok(cache) = vcrun_cache_dir() else {
        return;
    };
    let _ = copy_vcrun_dlls(&system32_dir(prefix), &cache);
}

fn seed_vcrun_from_cache(prefix: &Path) -> bool {
    let Ok(cache) = vcrun_cache_dir() else {
        return false;
    };
    if !vcrun_files_present(&cache) {
        return false;
    }
    copy_vcrun_dlls(&cache, &system32_dir(prefix))
}

fn vcrun_marked(prefix: &Path) -> bool {
    system32_dir(prefix).join(VCRUN_MARKER).is_file()
}

fn mark_vcrun(prefix: &Path) {
    let _ = fs::write(
        system32_dir(prefix).join(VCRUN_MARKER),
        b"microsoft-vcrun-2015-2022\n",
    );
}

fn clear_vcrun_stub_registry(wine64: &str, prefix: &Path) {
    for key in [
        r"HKLM\Software\Microsoft\VisualStudio\14.0\VC\Runtimes\X64",
        r"HKLM\Software\Wow6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\X64",
        r"HKLM\Software\Microsoft\VisualStudio\14.0\VC\Runtimes\x64",
    ] {
        let _ = Command::new(wine64)
            .env("WINEPREFIX", prefix)
            .env("WINEDEBUG", "-all")
            .args(["reg", "delete", key, "/f"])
            .status();
    }
}

fn run_vcrun_installer(
    registry: &LaunchRegistry,
    game_id: &str,
    wine64: &str,
    prefix: &Path,
    installer: &Path,
    extra_args: &[&str],
) -> Result<std::process::ExitStatus, CommandError> {
    let mut command = Command::new(wine64);
    command
        .env("WINEPREFIX", prefix)
        .env("WINEDEBUG", "-all")
        .env("WINEDLLOVERRIDES", "winemenubuilder.exe=d")
        .arg(installer)
        .args(extra_args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let child = command.spawn().map_err(|err| {
        CommandError::new(
            "prefix_failed",
            format!("Falha ao iniciar o instalador do Visual C++: {err}"),
        )
    })?;
    wait_child(
        registry,
        game_id,
        child,
        VCRUN_TIMEOUT,
        "O instalador do Visual C++ excedeu 5 minutos e foi interrompido.",
    )
}

fn apply_vcrun_overrides(wine64: &str, prefix: &std::path::Path) {
    for dll in VCRUN_DLLS {
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
}

fn ensure_vcrun_installer(
    registry: &LaunchRegistry,
    game_id: &str,
) -> Result<std::path::PathBuf, CommandError> {
    let cache = ensure_ponte_dir()?.join("cache");
    std::fs::create_dir_all(&cache).map_err(|err| {
        CommandError::new(
            "prefix_failed",
            format!("Não foi possível criar a pasta de cache: {err}"),
        )
    })?;
    let dest = cache.join("vc_redist.x64.exe");
    if dest.is_file() {
        if let Ok(meta) = dest.metadata() {
            if meta.len() > 1_000_000 {
                return Ok(dest);
            }
        }
    }
    append_log(
        registry,
        game_id,
        "A descarregar Visual C++ 2015-2022 (Microsoft)…",
    );
    let dest_str = dest.to_string_lossy().into_owned();
    let status = Command::new("curl")
        .args(["-fsSL", "--retry", "3", "-o", &dest_str, VCRUN_URL])
        .status()
        .map_err(|err| {
            CommandError::new(
                "prefix_failed",
                format!("Falha ao descarregar o Visual C++: {err}"),
            )
        })?;
    if !status.success() || !dest.is_file() {
        return Err(CommandError::new(
            "prefix_failed",
            "Não foi possível descarregar o Visual C++ da Microsoft.",
        ));
    }
    Ok(dest)
}

fn wait_child(
    registry: &LaunchRegistry,
    game_id: &str,
    mut child: Child,
    timeout: Duration,
    timeout_message: &str,
) -> Result<std::process::ExitStatus, CommandError> {
    let stderr_pipe = child.stderr.take();
    let stderr_thread = std::thread::spawn(move || {
        let mut buf = String::new();
        if let Some(mut pipe) = stderr_pipe {
            let _ = pipe.read_to_string(&mut buf);
        }
        buf
    });
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if started.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = stderr_thread.join();
                    return Err(CommandError::new("prefix_failed", timeout_message));
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(err) => {
                return Err(CommandError::new(
                    "prefix_failed",
                    format!("Falha ao consultar o processo: {err}"),
                ));
            }
        }
    };
    let stderr = stderr_thread.join().unwrap_or_default();
    if !stderr.trim().is_empty() {
        append_log(registry, game_id, stderr.trim());
    }
    Ok(status)
}

pub fn install_vcrun(registry: &LaunchRegistry, game: &Game) -> Result<(), CommandError> {
    let has_child = registry
        .processes
        .lock()
        .expect("process mutex")
        .contains_key(&game.id);
    if has_child {
        let _ = stop_game(registry, &game.id);
    }
    let runtime = resolve_wine(&game.runtime_kind).ok_or_else(|| {
        CommandError::new(
            "runtime_missing",
            "Nenhum wine64 encontrado para instalar o Visual C++.",
        )
    })?;
    let prefix = expand_home(&game.prefix_path);
    fs::create_dir_all(&prefix).ok();
    if vcrun_is_installed(&prefix) {
        capture_vcrun_to_cache(&prefix);
        if !vcrun_marked(&prefix) {
            apply_vcrun_overrides(&runtime.wine64_path, &prefix);
            mark_vcrun(&prefix);
        }
        append_log(
            registry,
            &game.id,
            "Visual C++ nativo já está neste prefixo. Nada a instalar.",
        );
        return Ok(());
    }
    if seed_vcrun_from_cache(&prefix) {
        apply_vcrun_overrides(&runtime.wine64_path, &prefix);
        mark_vcrun(&prefix);
        append_log(
            registry,
            &game.id,
            "Visual C++ copiado da cache do Ponte. O instalador da Microsoft não corre outra vez.",
        );
        return Ok(());
    }
    append_log(
        registry,
        &game.id,
        "O prefixo ainda não tem o Visual C++ oficial da Microsoft (as DLLs do Wine não chegam).",
    );
    clear_vcrun_stub_registry(&runtime.wine64_path, &prefix);
    let installer = ensure_vcrun_installer(registry, &game.id)?;
    append_log(
        registry,
        &game.id,
        "A instalar Microsoft Visual C++ 2015-2022 (x64) neste prefixo…",
    );
    let status = run_vcrun_installer(
        registry,
        &game.id,
        &runtime.wine64_path,
        &prefix,
        &installer,
        &["/install", "/quiet", "/norestart"],
    )?;
    if !vcrun_is_installed(&prefix) && status.code() == Some(1638) {
        append_log(
            registry,
            &game.id,
            "O instalador achou que o runtime já existia (falso). A desinstalar o stub e a repetir…",
        );
        let _ = run_vcrun_installer(
            registry,
            &game.id,
            &runtime.wine64_path,
            &prefix,
            &installer,
            &["/uninstall", "/quiet", "/norestart"],
        );
        clear_vcrun_stub_registry(&runtime.wine64_path, &prefix);
        let _ = run_vcrun_installer(
            registry,
            &game.id,
            &runtime.wine64_path,
            &prefix,
            &installer,
            &["/install", "/quiet", "/norestart"],
        );
    } else if !vcrun_is_installed(&prefix) && vcrun_installer_ok(&status) {
        append_log(
            registry,
            &game.id,
            "O instalador saiu bem, mas as DLLs nativas não apareceram. A tentar /repair…",
        );
        let _ = run_vcrun_installer(
            registry,
            &game.id,
            &runtime.wine64_path,
            &prefix,
            &installer,
            &["/repair", "/quiet", "/norestart"],
        );
    }
    if !vcrun_is_installed(&prefix) {
        return Err(CommandError::new(
            "prefix_failed",
            "O Visual C++ oficial não ficou neste prefixo. As DLLs continuam a ser as do Wine. Tente de novo em Instalar Visual C++.",
        ));
    }
    apply_vcrun_overrides(&runtime.wine64_path, &prefix);
    capture_vcrun_to_cache(&prefix);
    mark_vcrun(&prefix);
    append_log(
        registry,
        &game.id,
        "Visual C++ nativo instalado e guardado na cache. Os próximos prefixos reutilizam estas DLLs.",
    );
    Ok(())
}

pub fn launch_game(
    registry: Arc<LaunchRegistry>,
    game: &Game,
) -> Result<LaunchAttempt, CommandError> {
    let state = current_state(registry.as_ref(), &game.id);
    if matches!(state, LaunchState::Running | LaunchState::Starting) {
        if let Some(attempt_id) = active_attempt(registry.as_ref(), &game.id) {
            return attempts::read(&game.id, &attempt_id);
        }
        return Err(CommandError::new(
            "launch_busy",
            "Este jogo já tem uma tentativa de lançamento em andamento.",
        ));
    }
    if game.exe_path.trim().is_empty() || !is_windows_exe_path(&game.exe_path) {
        return Err(CommandError::new(
            "invalid_exe",
            "O caminho deve apontar para um ficheiro .exe.",
        ));
    }
    let exe = expand_home(&game.exe_path);
    if !exe.is_file() {
        return Err(CommandError::new(
            "invalid_exe",
            format!("Ficheiro não encontrado: {}", exe.display()),
        ));
    }
    if exe
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(is_helper_exe)
    {
        return Err(CommandError::new(
            "invalid_exe",
            "Este ficheiro não é o jogo. É uma ferramenta auxiliar (verificador, instalador ou redistributable). Adicione de novo a pasta do jogo para o Ponte escolher o .exe certo.",
        ));
    }
    let runtime = resolve_wine(&game.runtime_kind).ok_or_else(|| {
        CommandError::new(
            "runtime_missing",
            "Nenhum wine64 disponível. Instale o GPTK ou o Wine Gcenx e aponte o binário em Ajustes.",
        )
    })?;
    let settings = read_settings()?;
    let prefix = expand_home(&game.prefix_path);
    std::fs::create_dir_all(&prefix).ok();
    let preflight = crate::preflight::inspect(game);
    if preflight.status == crate::types::PreflightStatus::Blocked {
        let reason = preflight
            .checks
            .iter()
            .find(|check| check.status == crate::types::CheckStatus::Fail)
            .map(|check| check.detail.clone())
            .unwrap_or_else(|| "O preflight bloqueou este lançamento.".to_string());
        return Err(CommandError::new("launch_blocked", reason));
    }

    let exe_args = game
        .args
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let started_at = now_epoch();
    let attempt = LaunchAttempt {
        id: Uuid::new_v4().to_string(),
        game_id: game.id.clone(),
        status: LaunchAttemptStatus::Starting,
        started_at: started_at.clone(),
        updated_at: started_at,
        finished_at: None,
        executable: exe.to_string_lossy().into_owned(),
        working_dir: exe
            .parent()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default(),
        args: exe_args.clone(),
        runtime_kind: game.runtime_kind,
        runtime_path: runtime.wine64_path.clone(),
        dx_backend: game.dx_backend,
        prefix_path: prefix.to_string_lossy().into_owned(),
        pid: None,
        exit_code: None,
        duration_ms: None,
        profile_id: Some(
            match preflight.engine_guess {
                crate::types::EngineGuess::Unity => "unity-d3d11",
                crate::types::EngineGuess::Unreal => "unreal-d3d11",
                crate::types::EngineGuess::Unknown => "portable-win64",
            }
            .to_string(),
        ),
        preflight_fingerprint: Some(preflight.fingerprint),
        log_path: None,
        failure: None,
    };
    attempts::save(&attempt)?;
    registry
        .active_attempts
        .lock()
        .expect("attempt mutex")
        .insert(game.id.clone(), attempt.id.clone());

    set_state(registry.as_ref(), &game.id, LaunchState::Starting);
    append_log(
        registry.as_ref(),
        &game.id,
        &format!("Motor: {} ({})", runtime.label, runtime.wine64_path),
    );
    append_log(
        registry.as_ref(),
        &game.id,
        &format!("Backend: {}", backend_label(&game.dx_backend)),
    );

    let mut cmd = Command::new(&runtime.wine64_path);
    cmd.env("WINEPREFIX", &prefix);
    for (key, value) in env_for_backend(&game.dx_backend, settings.metal_hud) {
        cmd.env(key, value);
    }
    if let Some(work_dir) = exe.parent() {
        cmd.current_dir(work_dir);
    }
    cmd.arg(&exe);
    for arg in &exe_args {
        cmd.arg(arg);
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(err) => {
            set_state(registry.as_ref(), &game.id, LaunchState::Error);
            let failure = LaunchFailure {
                code: "spawn_failed".to_string(),
                message: format!("Falha ao iniciar: {err}"),
                retryable: true,
            };
            append_log(registry.as_ref(), &game.id, &failure.message);
            finish_attempt(
                registry.as_ref(),
                &game.id,
                LaunchAttemptStatus::Failed,
                None,
                Some(failure.clone()),
            );
            return Err(CommandError::new("launch_failed", failure.message));
        }
    };
    let pid = child.id();
    pipe_output(Arc::clone(&registry), game.id.clone(), &mut child);
    registry
        .processes
        .lock()
        .expect("process mutex")
        .insert(game.id.clone(), child);
    set_state(registry.as_ref(), &game.id, LaunchState::Running);
    append_log(
        registry.as_ref(),
        &game.id,
        &format!("Iniciando {}", exe.display()),
    );
    let mut persisted = attempt;
    persisted.pid = Some(pid);
    persisted.status = LaunchAttemptStatus::Running;
    persisted.updated_at = now_epoch();
    attempts::save(&persisted)?;
    Ok(persisted)
}

pub fn stop_game(registry: &LaunchRegistry, game_id: &str) -> Result<(), CommandError> {
    let state = current_state(registry, game_id);
    if matches!(state, LaunchState::Idle) {
        return Ok(());
    }
    set_state(registry, game_id, LaunchState::Stopping);
    if let Some(mut child) = registry
        .processes
        .lock()
        .expect("process mutex")
        .remove(game_id)
    {
        let _ = child.kill();
        let _ = child.wait();
    }
    append_log(registry, game_id, "Processo encerrado.");
    finish_attempt(
        registry,
        game_id,
        LaunchAttemptStatus::Interrupted,
        None,
        Some(LaunchFailure {
            code: "stopped_by_user".to_string(),
            message: "O processo foi encerrado pelo utilizador.".to_string(),
            retryable: true,
        }),
    );
    set_state(registry, game_id, LaunchState::Idle);
    Ok(())
}

pub fn logs_for(registry: &LaunchRegistry, game_id: &str) -> GameLog {
    let lines = registry
        .logs
        .lock()
        .expect("logs mutex")
        .get(game_id)
        .cloned()
        .unwrap_or_default();
    if !lines.is_empty() {
        return GameLog {
            game_id: game_id.to_string(),
            lines,
            updated_at: now_stamp(),
        };
    }
    if let Ok(Some(attempt)) = attempts::latest(game_id) {
        if let Ok(log) = attempts::log(game_id, &attempt.id) {
            return log;
        }
    }
    GameLog {
        game_id: game_id.to_string(),
        lines,
        updated_at: now_stamp(),
    }
}

#[cfg(test)]
mod tests {
    use super::{vcrun_installer_ok, vcrun_is_installed};
    use std::fs;
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    fn status_from_code(code: i32) -> ExitStatus {
        ExitStatus::from_raw(code << 8)
    }

    #[test]
    fn vcrun_unix_exit_0_is_ok_and_1_is_not() {
        assert!(vcrun_installer_ok(&status_from_code(0)));
        assert!(!vcrun_installer_ok(&status_from_code(1)));
        assert!(!vcrun_installer_ok(&status_from_code(1638)));
    }

    #[test]
    fn wine_builtin_dlls_do_not_count_as_vcrun() {
        let dir = std::env::temp_dir().join(format!("ponte-vcrun-builtin-{}", std::process::id()));
        let system32 = dir.join("drive_c/windows/system32");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&system32).unwrap();
        for name in ["vcruntime140.dll", "vcruntime140_1.dll", "msvcp140.dll"] {
            fs::write(system32.join(name), b"header Wine builtin DLL tail").unwrap();
        }
        assert!(!vcrun_is_installed(&dir));
        fs::write(system32.join("vcruntime140.dll"), b"Microsoft Visual C++").unwrap();
        fs::write(system32.join("vcruntime140_1.dll"), b"Microsoft Visual C++").unwrap();
        fs::write(system32.join("msvcp140.dll"), b"Microsoft Visual C++").unwrap();
        assert!(vcrun_is_installed(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn copies_native_vcrun_dlls_between_prefixes() {
        let from = std::env::temp_dir().join(format!("ponte-vcrun-from-{}", std::process::id()));
        let to = std::env::temp_dir().join(format!("ponte-vcrun-to-{}", std::process::id()));
        let _ = fs::remove_dir_all(&from);
        let _ = fs::remove_dir_all(&to);
        fs::create_dir_all(&from).unwrap();
        for name in ["vcruntime140.dll", "vcruntime140_1.dll", "msvcp140.dll"] {
            fs::write(from.join(name), b"Microsoft Visual C++").unwrap();
        }
        assert!(super::copy_vcrun_dlls(
            &from,
            &to.join("drive_c/windows/system32")
        ));
        assert!(vcrun_is_installed(&to));
        let _ = fs::remove_dir_all(&from);
        let _ = fs::remove_dir_all(&to);
    }
}
