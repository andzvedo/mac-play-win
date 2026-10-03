use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::CommandError;
use crate::storage::ensure_ponte_dir;
use crate::types::{Game, GameLog, LaunchAttempt, LaunchAttemptStatus, LaunchFailure, RuntimeKind};

const MAX_ATTEMPTS: usize = 20;
const MAX_LOG_BYTES: usize = 5 * 1024 * 1024;

fn safe_segment(value: &str, label: &str) -> Result<(), CommandError> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(CommandError::new(
            "not_found",
            format!("{label} de lançamento inválido."),
        ));
    }
    Ok(())
}

fn attempts_root() -> Result<PathBuf, CommandError> {
    Ok(ensure_ponte_dir()?.join("attempts"))
}

fn attempt_dir(game_id: &str, attempt_id: &str) -> Result<PathBuf, CommandError> {
    safe_segment(game_id, "ID do jogo")?;
    safe_segment(attempt_id, "ID da tentativa")?;
    let dir = attempts_root()?.join(game_id).join(attempt_id);
    fs::create_dir_all(&dir).map_err(|error| {
        CommandError::new(
            "unknown",
            format!("Não foi possível criar o histórico: {error}"),
        )
    })?;
    Ok(dir)
}

fn write_atomic(path: &Path, content: &str) -> Result<(), CommandError> {
    let temp = path.with_extension("tmp");
    fs::write(&temp, content).map_err(|error| {
        CommandError::new("unknown", format!("Falha ao escrever histórico: {error}"))
    })?;
    fs::rename(&temp, path).map_err(|error| {
        CommandError::new("unknown", format!("Falha ao confirmar histórico: {error}"))
    })
}

fn json_path(game_id: &str, attempt_id: &str) -> Result<PathBuf, CommandError> {
    Ok(attempt_dir(game_id, attempt_id)?.join("attempt.json"))
}

fn existing_json_path(game_id: &str, attempt_id: &str) -> Result<PathBuf, CommandError> {
    safe_segment(game_id, "ID do jogo")?;
    safe_segment(attempt_id, "ID da tentativa")?;
    Ok(attempts_root()?
        .join(game_id)
        .join(attempt_id)
        .join("attempt.json"))
}

pub fn save(attempt: &LaunchAttempt) -> Result<(), CommandError> {
    let path = json_path(&attempt.game_id, &attempt.id)?;
    let content = serde_json::to_string_pretty(attempt)
        .map_err(|error| CommandError::new("unknown", error.to_string()))?;
    write_atomic(&path, &content)
}

pub fn record_failure(
    game: &Game,
    status: LaunchAttemptStatus,
    code: &str,
    message: &str,
    fingerprint: Option<String>,
) -> Result<LaunchAttempt, CommandError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());
    let executable = crate::storage::expand_home(&game.exe_path);
    let attempt = LaunchAttempt {
        id: uuid::Uuid::new_v4().to_string(),
        game_id: game.id.clone(),
        status,
        started_at: now.clone(),
        updated_at: now.clone(),
        finished_at: Some(now),
        executable: executable.to_string_lossy().into_owned(),
        working_dir: executable
            .parent()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default(),
        args: game.args.split_whitespace().map(str::to_string).collect(),
        runtime_kind: game.runtime_kind,
        runtime_path: String::new(),
        dx_backend: game.dx_backend,
        prefix_path: crate::storage::expand_home(&game.prefix_path)
            .to_string_lossy()
            .into_owned(),
        pid: None,
        exit_code: None,
        duration_ms: Some(0),
        profile_id: Some(
            match game.runtime_kind {
                RuntimeKind::Gptk => "gptk",
                RuntimeKind::GcenxStaging => "staging",
                RuntimeKind::Winehq => "winehq",
                RuntimeKind::Winecx => "winecx",
                RuntimeKind::Crossover => "crossover",
                RuntimeKind::Custom => "custom",
            }
            .to_string(),
        ),
        preflight_fingerprint: fingerprint,
        log_path: None,
        failure: Some(LaunchFailure {
            code: code.to_string(),
            message: message.to_string(),
            retryable: code != "invalid_exe" && code != "launch_blocked",
        }),
    };
    save(&attempt)?;
    Ok(attempt)
}

pub fn read(game_id: &str, attempt_id: &str) -> Result<LaunchAttempt, CommandError> {
    let path = existing_json_path(game_id, attempt_id)?;
    let content = fs::read_to_string(&path)
        .map_err(|_| CommandError::new("not_found", "Tentativa de lançamento não encontrada."))?;
    serde_json::from_str(&content)
        .map_err(|error| CommandError::new("unknown", format!("Histórico inválido: {error}")))
}

pub fn list(game_id: &str) -> Result<Vec<LaunchAttempt>, CommandError> {
    safe_segment(game_id, "ID do jogo")?;
    let root = attempts_root()?.join(game_id);
    let mut attempts = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return Ok(attempts);
    };
    for entry in entries.flatten() {
        let path = entry.path().join("attempt.json");
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        if let Ok(attempt) = serde_json::from_str::<LaunchAttempt>(&content) {
            attempts.push(attempt);
        }
    }
    attempts.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    attempts.truncate(MAX_ATTEMPTS);
    Ok(attempts)
}

pub fn latest(game_id: &str) -> Result<Option<LaunchAttempt>, CommandError> {
    Ok(list(game_id)?.into_iter().next())
}

pub fn append_log(attempt: &LaunchAttempt, line: &str) -> Result<(), CommandError> {
    let path = attempt_dir(&attempt.game_id, &attempt.id)?.join("launch.log");
    let mut content = fs::read_to_string(&path).unwrap_or_default();
    content.push_str(line);
    content.push('\n');
    if content.len() > MAX_LOG_BYTES {
        let minimum_start = content.len() - MAX_LOG_BYTES;
        let start = content
            .char_indices()
            .find(|(index, _)| *index >= minimum_start)
            .map(|(index, _)| index)
            .unwrap_or(0);
        content = content[start..].to_string();
    }
    fs::write(path, content)
        .map_err(|error| CommandError::new("unknown", format!("Falha ao gravar log: {error}")))
}

pub fn log(game_id: &str, attempt_id: &str) -> Result<GameLog, CommandError> {
    safe_segment(game_id, "ID do jogo")?;
    safe_segment(attempt_id, "ID da tentativa")?;
    let path = attempts_root()?
        .join(game_id)
        .join(attempt_id)
        .join("launch.log");
    let lines = fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .collect();
    Ok(GameLog {
        game_id: game_id.to_string(),
        lines,
        updated_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string()),
    })
}
