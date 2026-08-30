use std::path::{Path, PathBuf};

pub fn is_windows_exe_path(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return false;
    }
    let name = Path::new(trimmed)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(trimmed);
    name.to_ascii_lowercase().ends_with(".exe")
}

fn trim_trailing_slash(path: &Path) -> PathBuf {
    PathBuf::from(
        path.to_string_lossy()
            .trim_end_matches('/')
            .trim_end_matches('\\'),
    )
}

pub fn prefix_is_owned(prefix: &Path, prefixes_dir: &Path, game_id: &str) -> bool {
    if game_id.is_empty()
        || game_id == "."
        || game_id == ".."
        || game_id.contains('/')
        || game_id.contains('\\')
    {
        return false;
    }
    let prefixes_dir = trim_trailing_slash(prefixes_dir);
    let prefix = trim_trailing_slash(prefix);
    if prefixes_dir.components().count() < 3 {
        return false;
    }
    let Some(name) = prefix.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if name != game_id {
        return false;
    }
    prefix.parent() == Some(prefixes_dir.as_path())
}

#[cfg(test)]
mod tests {
    use super::{is_windows_exe_path, prefix_is_owned};
    use std::path::Path;

    #[test]
    fn accepts_windows_exe_suffix() {
        assert!(is_windows_exe_path("/Games/Hades/Hades.exe"));
        assert!(is_windows_exe_path("C:\\Games\\Hades.EXE"));
        assert!(is_windows_exe_path("  /tmp/jogo.exe  "));
    }

    #[test]
    fn rejects_non_exe_paths() {
        assert!(!is_windows_exe_path(""));
        assert!(!is_windows_exe_path("/Games/Hades/Hades.app"));
        assert!(!is_windows_exe_path("/Games/Hades/Hades.exe.bak"));
        assert!(!is_windows_exe_path("exe"));
    }

    #[test]
    fn owned_prefix_must_live_under_prefixes_dir() {
        let dir = Path::new("/tmp/ponte/prefixes");
        assert!(prefix_is_owned(
            Path::new("/tmp/ponte/prefixes/abc-id"),
            dir,
            "abc-id",
        ));
        assert!(prefix_is_owned(
            Path::new("/tmp/ponte/prefixes/abc-id/"),
            Path::new("/tmp/ponte/prefixes/"),
            "abc-id",
        ));
        assert!(!prefix_is_owned(
            Path::new("/tmp/ponte/prefixes/abc-id"),
            dir,
            "other",
        ));
        assert!(!prefix_is_owned(
            Path::new("/etc/abc-id"),
            dir,
            "abc-id",
        ));
        assert!(!prefix_is_owned(
            Path::new("/tmp/ponte/prefixes/abc-id"),
            Path::new("/"),
            "abc-id",
        ));
    }
}
