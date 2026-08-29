use std::fs;
use std::path::{Path, PathBuf};

use crate::error::CommandError;
use crate::types::{LibraryFile, Settings};

pub fn ponte_dir() -> Result<PathBuf, CommandError> {
    let data = dirs::data_dir().ok_or_else(|| {
        CommandError::new("unknown", "Não foi possível resolver Application Support.")
    })?;
    Ok(data.join("Ponte"))
}

pub fn ensure_ponte_dir() -> Result<PathBuf, CommandError> {
    let dir = ponte_dir()?;
    fs::create_dir_all(&dir).map_err(|err| {
        CommandError::new("unknown", format!("Não foi possível criar a pasta Ponte: {err}"))
    })?;
    fs::create_dir_all(dir.join("prefixes")).ok();
    Ok(dir)
}

pub fn library_path() -> Result<PathBuf, CommandError> {
    Ok(ensure_ponte_dir()?.join("library.json"))
}

pub fn settings_path() -> Result<PathBuf, CommandError> {
    Ok(ensure_ponte_dir()?.join("settings.json"))
}

fn write_atomic(path: &Path, contents: &str) -> Result<(), CommandError> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, contents).map_err(|err| {
        CommandError::new("unknown", format!("Falha ao gravar: {err}"))
    })?;
    fs::rename(&tmp, path).map_err(|err| {
        CommandError::new("unknown", format!("Falha ao gravar: {err}"))
    })?;
    Ok(())
}

pub fn read_library() -> Result<LibraryFile, CommandError> {
    let path = library_path()?;
    if !path.exists() {
        return Ok(LibraryFile {
            version: 1,
            games: vec![],
        });
    }
    let raw = fs::read_to_string(&path).map_err(|err| {
        CommandError::new("unknown", format!("Falha ao ler a biblioteca: {err}"))
    })?;
    match serde_json::from_str::<LibraryFile>(&raw) {
        Ok(file) if file.version == 1 => Ok(file),
        _ => Ok(LibraryFile {
            version: 1,
            games: vec![],
        }),
    }
}

pub fn write_library(library: &LibraryFile) -> Result<(), CommandError> {
    let path = library_path()?;
    let raw = serde_json::to_string_pretty(library)
        .map_err(|err| CommandError::new("unknown", err.to_string()))?;
    write_atomic(&path, &raw)
}

pub fn read_settings() -> Result<Settings, CommandError> {
    let path = settings_path()?;
    if !path.exists() {
        return Ok(Settings::default());
    }
    let raw = fs::read_to_string(&path).map_err(|err| {
        CommandError::new("unknown", format!("Falha ao ler ajustes: {err}"))
    })?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

pub fn write_settings(settings: &Settings) -> Result<(), CommandError> {
    let path = settings_path()?;
    let raw = serde_json::to_string_pretty(settings)
        .map_err(|err| CommandError::new("unknown", err.to_string()))?;
    write_atomic(&path, &raw)
}

pub fn expand_home(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    PathBuf::from(path)
}
