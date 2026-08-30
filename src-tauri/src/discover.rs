use std::fs;
use std::path::{Path, PathBuf};

use crate::error::CommandError;
use crate::paths::is_windows_exe_path;
use crate::setup::{guess_engine, recommended_stack_for_engine};
use crate::storage::expand_home;
use crate::types::GameDiscovery;

const MAX_DEPTH: usize = 8;
const MAX_DIRS: usize = 2_500;
const MAX_EXES: usize = 400;

struct Candidate {
    path: PathBuf,
    size: u64,
    score: i32,
}

pub fn resolve_game_path(path: &str) -> Result<GameDiscovery, CommandError> {
    let trimmed = path
        .trim()
        .trim_end_matches('/')
        .trim_end_matches('\\')
        .to_string();
    if trimmed.is_empty() {
        return Err(CommandError::new(
            "invalid_folder",
            "Escolha a pasta onde o jogo está no disco.",
        ));
    }
    let expanded = expand_home(&trimmed);
    if is_root_path(&expanded) {
        return Err(CommandError::new(
            "invalid_folder",
            "Escolha a pasta de um jogo, não a raiz do disco.",
        ));
    }
    if !expanded.exists() {
        return Err(CommandError::new(
            "invalid_folder",
            format!("Pasta não encontrada: {}", expanded.display()),
        ));
    }
    if expanded.is_file() {
        return discovery_from_exe(&expanded);
    }
    if !expanded.is_dir() {
        return Err(CommandError::new(
            "invalid_folder",
            "Escolha a pasta do jogo.",
        ));
    }
    if looks_like_library_root(&expanded) {
        return Err(CommandError::new(
            "invalid_folder",
            "Esta pasta parece ter vários jogos. Escolha a pasta de um só título.",
        ));
    }
    let exe = pick_exe_in_dir(&expanded)?;
    Ok(discovery_from_paths(&expanded, &exe))
}

fn discovery_from_exe(exe: &Path) -> Result<GameDiscovery, CommandError> {
    if !is_windows_exe_path(&exe.to_string_lossy()) {
        return Err(CommandError::new(
            "invalid_folder",
            "Escolha a pasta do jogo. O Ponte encontra o .exe.",
        ));
    }
    let folder = exe.parent().unwrap_or(exe);
    Ok(discovery_from_paths(folder, exe))
}

fn discovery_from_paths(folder: &Path, exe: &Path) -> GameDiscovery {
    let engine = guess_engine(exe);
    let (runtime, backend) = recommended_stack_for_engine(&engine);
    GameDiscovery {
        folder_path: folder.to_string_lossy().into_owned(),
        exe_path: exe.to_string_lossy().into_owned(),
        suggested_name: suggested_name(folder, exe),
        engine_guess: engine,
        recommended_runtime: runtime,
        recommended_backend: backend,
    }
}

fn is_root_path(path: &Path) -> bool {
    path.parent().is_none() || path.as_os_str() == "/"
}

fn skip_dir_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower.starts_with('.') {
        return true;
    }
    matches!(
        lower.as_str(),
        "engine"
            | "_commonredist"
            | "__installer"
            | "redist"
            | "directx"
            | "directxredist"
            | "dotnet"
            | "node_modules"
            | "__macosx"
            | "payloads"
            | "intermediate"
            | "saved"
            | "easyanticheat"
            | "battleye"
            | "easyanticheat_eos"
    )
}

pub fn is_helper_exe(file_name: &str) -> bool {
    let lower = file_name.to_ascii_lowercase();
    let stem = lower.strip_suffix(".exe").unwrap_or(&lower);
    const NEEDLES: &[&str] = &[
        "unins",
        "uninstall",
        "uninst",
        "vcredist",
        "vc_redist",
        "dxsetup",
        "dxwebsetup",
        "unitycrashhandler",
        "crashhandler",
        "crashreporter",
        "crashpad",
        "crashreportclient",
        "easyanticheat",
        "battleye",
        "beservice",
        "unrealcefsubprocess",
        "unrealeditor",
        "swarmagent",
        "notification_helper",
        "steamerrorreporter",
        "dotnetfx",
        "windowsdesktop-runtime",
        "prereq",
        "ue4prereq",
        "ue5prereq",
    ];
    if NEEDLES.iter().any(|needle| stem.contains(needle)) {
        return true;
    }
    matches!(
        stem,
        "setup" | "install" | "installer" | "repair" | "redist" | "dxsetup"
    )
}

fn normalize_label(value: &str) -> String {
    value
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn prettify(name: &str) -> String {
    let stripped = name
        .trim()
        .trim_end_matches("-Win64-Shipping")
        .trim_end_matches("-Win32-Shipping")
        .trim_end_matches("-Win64")
        .trim_end_matches("-Win32");
    let spaced = stripped.replace(['_', '-'], " ");
    spaced.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn suggested_name(folder: &Path, exe: &Path) -> String {
    let folder_name = folder
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let folder_lower = folder_name.to_ascii_lowercase();
    let generic_folder = matches!(
        folder_lower.as_str(),
        "win64" | "win32" | "binaries" | "bin" | "game" | "games"
    );
    if generic_folder {
        if let Some(stem) = exe.file_stem().and_then(|name| name_from_stem(name)) {
            return stem;
        }
    }
    if !folder_name.is_empty() {
        return prettify(folder_name);
    }
    exe.file_stem()
        .and_then(name_from_stem)
        .unwrap_or_else(|| "Jogo".to_string())
}

fn name_from_stem(stem: &std::ffi::OsStr) -> Option<String> {
    let raw = stem.to_str()?;
    let pretty = prettify(raw);
    if pretty.is_empty() {
        None
    } else {
        Some(pretty)
    }
}

fn dir_has_unity_data(dir: &Path) -> bool {
    if dir.join("UnityPlayer.dll").is_file() || dir.join("GameAssembly.dll").is_file() {
        return true;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let path = entry.path();
        path.is_dir()
            && entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.to_ascii_lowercase().ends_with("_data"))
    })
}

fn dir_has_named_exe(dir: &Path) -> bool {
    let Some(name) = dir.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    dir.join(format!("{name}.exe")).is_file()
}

fn dir_looks_like_game(dir: &Path) -> bool {
    if skip_dir_name(
        dir.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(""),
    ) {
        return false;
    }
    dir_has_unity_data(dir) || dir_has_named_exe(dir) || shipping_under(dir)
}

fn shipping_under(dir: &Path) -> bool {
    if first_shipping_in(&dir.join("Binaries/Win64")).is_some() {
        return true;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let path = entry.path();
        path.is_dir() && first_shipping_in(&path.join("Binaries/Win64")).is_some()
    })
}

fn first_shipping_in(dir: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    entries.flatten().map(|entry| entry.path()).find(|path| {
        path.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    let lower = name.to_ascii_lowercase();
                    lower.ends_with(".exe") && lower.contains("win64-shipping")
                })
    })
}

fn looks_like_library_root(root: &Path) -> bool {
    let Ok(entries) = fs::read_dir(root) else {
        return false;
    };
    let child_games = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && dir_looks_like_game(path))
        .count();
    if child_games < 2 {
        return false;
    }
    let root_is_game = dir_has_unity_data(root)
        || root.read_dir().ok().is_some_and(|entries| {
            entries.flatten().any(|entry| {
                let path = entry.path();
                path.is_file()
                    && is_windows_exe_path(&path.to_string_lossy())
                    && !is_helper_exe(
                        path.file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or(""),
                    )
            })
        })
        || shipping_under(root);
    !root_is_game
}

fn score_exe(
    path: &Path,
    root: &Path,
    root_name: &str,
    size: u64,
    siblings: &[String],
) -> i32 {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let stem = Path::new(&name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_string();
    let rel = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let rel_lower = rel.to_ascii_lowercase();
    let mut score = 0i32;

    if name.contains("win64-shipping") {
        score += 1_000;
    }
    if rel_lower.contains("binaries/win64") {
        score += 250;
    }

    let data_dir = format!("{stem}_data");
    if siblings.iter().any(|item| item == &data_dir) {
        score += 500;
    }
    if siblings.iter().any(|item| item == "unityplayer.dll") {
        score += 120;
    }
    if siblings.iter().any(|item| item == "gameassembly.dll") {
        score += 80;
    }

    let folder_norm = normalize_label(root_name);
    let stem_norm = normalize_label(
        stem.trim_end_matches("-win64-shipping")
            .trim_end_matches("-win32-shipping")
            .trim_end_matches("-win64")
            .trim_end_matches("-win32"),
    );
    if !folder_norm.is_empty()
        && !stem_norm.is_empty()
        && (stem_norm == folder_norm
            || folder_norm.contains(&stem_norm)
            || stem_norm.contains(&folder_norm))
    {
        score += 200;
    }

    if size > 50_000_000 {
        score += 150;
    } else if size > 10_000_000 {
        score += 100;
    } else if size > 1_000_000 {
        score += 50;
    } else if size < 80_000 {
        score -= 80;
    }

    let depth = rel_lower.matches('/').count();
    score -= (depth as i32) * 12;

    if rel_lower.contains("win32") && !rel_lower.contains("win64") {
        score -= 80;
    }
    if stem.contains("launcher") && !name.contains("win64-shipping") {
        score -= 50;
    }
    if stem.contains("editor") {
        score -= 400;
    }

    score
}

fn pick_exe_in_dir(root: &Path) -> Result<PathBuf, CommandError> {
    let root_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    let mut dirs_seen = 0usize;

    while let Some((dir, depth)) = stack.pop() {
        dirs_seen += 1;
        if dirs_seen > MAX_DIRS || candidates.len() >= MAX_EXES {
            break;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        let mut child_dirs = Vec::new();
        let mut files = Vec::new();
        let mut siblings = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            siblings.push(name);
            if meta.is_dir() {
                child_dirs.push(path);
            } else if meta.is_file() {
                files.push((path, meta.len()));
            }
        }
        for (path, size) in files {
            if candidates.len() >= MAX_EXES {
                break;
            }
            if !is_windows_exe_path(&path.to_string_lossy()) {
                continue;
            }
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if is_helper_exe(file_name) {
                continue;
            }
            let score = score_exe(&path, root, root_name, size, &siblings);
            candidates.push(Candidate { path, size, score });
        }
        if depth >= MAX_DEPTH {
            continue;
        }
        for child in child_dirs {
            let name = child
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if skip_dir_name(name) {
                continue;
            }
            stack.push((child, depth + 1));
        }
    }

    if candidates.is_empty() {
        return Err(CommandError::new(
            "invalid_folder",
            "Não foi encontrado um .exe de jogo nesta pasta (só instaladores ou desinstaladores não servem).",
        ));
    }

    candidates.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(b.size.cmp(&a.size))
            .then_with(|| a.path.cmp(&b.path))
    });
    Ok(candidates.remove(0).path)
}

#[cfg(test)]
mod tests {
    use super::{is_helper_exe, resolve_game_path};
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ponte-discover-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn touch(path: &PathBuf) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, b"exe").unwrap();
    }

    #[test]
    fn helpers_are_skipped() {
        assert!(is_helper_exe("unins000.exe"));
        assert!(is_helper_exe("UnityCrashHandler64.exe"));
        assert!(is_helper_exe("VC_redist.x64.exe"));
        assert!(is_helper_exe("setup.exe"));
        assert!(!is_helper_exe("Hades.exe"));
        assert!(!is_helper_exe("PenguinHotel-Win64-Shipping.exe"));
    }

    #[test]
    fn picks_unity_over_crash_handler() {
        let dir = temp_dir("unity");
        fs::create_dir_all(dir.join("REPO_Data")).unwrap();
        touch(&dir.join("REPO.exe"));
        touch(&dir.join("UnityCrashHandler64.exe"));
        let found = resolve_game_path(&dir.to_string_lossy()).unwrap();
        assert!(found.exe_path.ends_with("REPO.exe"));
        assert_eq!(found.engine_guess, crate::types::EngineGuess::Unity);
    }

    #[test]
    fn picks_unreal_shipping_over_bootstrap() {
        let dir = temp_dir("unreal");
        let root = dir.join("PenguinHotel");
        touch(&root.join("PenguinHotel.exe"));
        fs::create_dir_all(root.join("Engine")).unwrap();
        let shipping = root.join("Chameleon/Binaries/Win64/PenguinHotel-Win64-Shipping.exe");
        touch(&shipping);
        let found = resolve_game_path(&root.to_string_lossy()).unwrap();
        assert_eq!(found.exe_path, shipping.to_string_lossy());
        assert_eq!(found.engine_guess, crate::types::EngineGuess::Unreal);
        assert_eq!(found.suggested_name, "PenguinHotel");
    }

    #[test]
    fn rejects_folder_with_several_games() {
        let dir = temp_dir("common");
        fs::create_dir_all(dir.join("Hades/Hades_Data")).unwrap();
        touch(&dir.join("Hades/Hades.exe"));
        fs::create_dir_all(dir.join("Celeste/Celeste_Data")).unwrap();
        touch(&dir.join("Celeste/Celeste.exe"));
        let err = resolve_game_path(&dir.to_string_lossy()).unwrap_err();
        assert_eq!(err.code, "invalid_folder");
        assert!(err.message.contains("vários jogos"));
    }

    #[test]
    fn rejects_folder_with_only_uninstallers() {
        let dir = temp_dir("emptyish");
        touch(&dir.join("unins000.exe"));
        touch(&dir.join("setup.exe"));
        let err = resolve_game_path(&dir.to_string_lossy()).unwrap_err();
        assert_eq!(err.code, "invalid_folder");
    }

    #[test]
    fn accepts_a_direct_exe_path() {
        let dir = temp_dir("direct");
        let exe = dir.join("Hades.exe");
        touch(&exe);
        let found = resolve_game_path(&exe.to_string_lossy()).unwrap();
        assert_eq!(found.exe_path, exe.to_string_lossy());
    }
}
