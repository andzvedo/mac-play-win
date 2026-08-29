use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::env_backend::env_for_backend;
use crate::error::CommandError;
use crate::runtimes::resolve_wine;
use crate::storage::{expand_home, read_settings};
use crate::types::{Game, GameLog, LaunchState};

const LOG_CAP: usize = 500;

pub struct LaunchRegistry {
    pub processes: Mutex<HashMap<String, Child>>,
    pub logs: Mutex<HashMap<String, Vec<String>>>,
    pub states: Mutex<HashMap<String, LaunchState>>,
}

impl LaunchRegistry {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
            logs: Mutex::new(HashMap::new()),
            states: Mutex::new(HashMap::new()),
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
    let mut logs = registry.logs.lock().expect("logs mutex");
    let lines = logs.entry(game_id.to_string()).or_default();
    lines.push(format!("[{}] {message}", now_stamp()));
    if lines.len() > LOG_CAP {
        let extra = lines.len() - LOG_CAP;
        lines.drain(0..extra);
    }
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
                } else {
                    set_state(registry, game_id, LaunchState::Error);
                    append_log(
                        registry,
                        game_id,
                        &format!("Processo saiu com código {status}."),
                    );
                }
            }
            Ok(None) => {}
            Err(err) => {
                processes.remove(game_id);
                set_state(registry, game_id, LaunchState::Error);
                append_log(registry, game_id, &format!("Falha ao consultar processo: {err}"));
            }
        }
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

pub fn create_prefix(registry: &LaunchRegistry, game: &Game) -> Result<(), CommandError> {
    let runtime = resolve_wine(&game.runtime_kind).ok_or_else(|| {
        CommandError::new(
            "runtime_missing",
            "Nenhum wine64 encontrado para criar o prefixo.",
        )
    })?;
    let prefix = expand_home(&game.prefix_path);
    std::fs::create_dir_all(&prefix).ok();
    append_log(
        registry,
        &game.id,
        &format!("WINEPREFIX={} wineboot --init", prefix.display()),
    );
    let output = Command::new(&runtime.wine64_path)
        .env("WINEPREFIX", &prefix)
        .args(["wineboot", "--init"])
        .output()
        .map_err(|err| {
            CommandError::new("prefix_failed", format!("wineboot falhou: {err}"))
        })?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.trim().is_empty() {
        append_log(registry, &game.id, stderr.trim());
    }
    if !output.status.success() {
        return Err(CommandError::new(
            "prefix_failed",
            format!("wineboot saiu com {}", output.status),
        ));
    }
    append_log(registry, &game.id, "Prefixo pronto.");
    Ok(())
}

pub fn launch_game(
    registry: Arc<LaunchRegistry>,
    game: &Game,
) -> Result<(), CommandError> {
    let state = current_state(registry.as_ref(), &game.id);
    if matches!(state, LaunchState::Running | LaunchState::Starting) {
        return Ok(());
    }
    if game.exe_path.trim().is_empty() {
        return Err(CommandError::new("invalid_exe", "Caminho do .exe está vazio."));
    }
    let exe = expand_home(&game.exe_path);
    if !exe.is_file() {
        return Err(CommandError::new(
            "invalid_exe",
            format!("Ficheiro não encontrado: {}", exe.display()),
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

    set_state(registry.as_ref(), &game.id, LaunchState::Starting);
    append_log(
        registry.as_ref(),
        &game.id,
        &format!("Motor: {} ({})", runtime.label, runtime.wine64_path),
    );

    let mut cmd = Command::new(&runtime.wine64_path);
    cmd.env("WINEPREFIX", &prefix);
    for (key, value) in env_for_backend(&game.dx_backend, settings.metal_hud) {
        cmd.env(key, value);
    }
    cmd.arg(&exe);
    for arg in game.args.split_whitespace() {
        cmd.arg(arg);
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|err| {
        set_state(registry.as_ref(), &game.id, LaunchState::Error);
        CommandError::new("launch_failed", format!("Falha ao iniciar: {err}"))
    })?;
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
    Ok(())
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
    GameLog {
        game_id: game_id.to_string(),
        lines,
        updated_at: now_stamp(),
    }
}
