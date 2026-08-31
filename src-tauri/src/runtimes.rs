use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::storage::{expand_home, read_settings};
use crate::types::{DxBackend, Runtime, RuntimeKind};

struct CatalogEntry {
    kind: RuntimeKind,
    label: &'static str,
    backends: &'static [DxBackend],
    install_url: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeCandidate {
    kind: RuntimeKind,
    path: PathBuf,
}

const CATALOG: &[CatalogEntry] = &[
    CatalogEntry {
        kind: RuntimeKind::Gptk,
        label: "Game Porting Toolkit",
        backends: &[DxBackend::D3dmetal, DxBackend::Wined3d],
        install_url: Some("https://developer.apple.com/games/game-porting-toolkit/"),
    },
    CatalogEntry {
        kind: RuntimeKind::GcenxStaging,
        label: "Wine Staging (Gcenx)",
        backends: &[DxBackend::Dxmt, DxBackend::Dxvk, DxBackend::Wined3d],
        install_url: Some("https://github.com/Gcenx/macOS_Wine_builds"),
    },
    CatalogEntry {
        kind: RuntimeKind::Winehq,
        label: "WineHQ Stable",
        backends: &[DxBackend::Wined3d, DxBackend::Dxvk],
        install_url: Some("https://wiki.winehq.org/MacOS"),
    },
    CatalogEntry {
        kind: RuntimeKind::Winecx,
        label: "Wine CX (Gcenx)",
        backends: &[DxBackend::Dxvk, DxBackend::Dxmt, DxBackend::Wined3d],
        install_url: Some("https://github.com/Gcenx/winecx"),
    },
    CatalogEntry {
        kind: RuntimeKind::Crossover,
        label: "CrossOver",
        backends: &[
            DxBackend::D3dmetal,
            DxBackend::Dxmt,
            DxBackend::Dxvk,
            DxBackend::Wined3d,
        ],
        install_url: Some("https://www.codeweavers.com/crossover"),
    },
    CatalogEntry {
        kind: RuntimeKind::Custom,
        label: "wine64 manual",
        backends: &[
            DxBackend::D3dmetal,
            DxBackend::Dxmt,
            DxBackend::Dxvk,
            DxBackend::Wined3d,
        ],
        install_url: None,
    },
];

fn normalized_component(component: &std::ffi::OsStr) -> Option<String> {
    component.to_str().map(|value| {
        value
            .to_ascii_lowercase()
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .collect()
    })
}

fn path_has_component(path: &Path, expected: &str) -> bool {
    path.components()
        .any(|component| normalized_component(component.as_os_str()).as_deref() == Some(expected))
}

fn classify_runtime_path(path: &Path) -> Option<RuntimeKind> {
    // Specific application/container names take precedence over generic
    // parent directories. WineCX may contain "crossover" in its name, but a
    // CrossOver.app container is unambiguously CrossOver.
    if path_has_component(path, "crossoverapp")
        || path_has_component(path, "crossover")
        || path_has_component(path, "codeweavers")
    {
        return Some(RuntimeKind::Crossover);
    }
    if path.components().any(|component| {
        let Some(component) = normalized_component(component.as_os_str()) else {
            return false;
        };
        component.starts_with("gameportingtoolkit")
    }) {
        return Some(RuntimeKind::Gptk);
    }
    if path.components().any(|component| {
        let Some(component) = normalized_component(component.as_os_str()) else {
            return false;
        };
        component.starts_with("winecx") || component.starts_with("winecrossover")
    }) {
        return Some(RuntimeKind::Winecx);
    }
    if path.components().any(|component| {
        let Some(component) = normalized_component(component.as_os_str()) else {
            return false;
        };
        component.starts_with("winestaging") || component == "gcenx"
    }) {
        return Some(RuntimeKind::GcenxStaging);
    }
    if path.components().any(|component| {
        let Some(component) = normalized_component(component.as_os_str()) else {
            return false;
        };
        component.starts_with("winestable")
            || component.starts_with("winedevel")
            || component == "winehq"
    }) {
        return Some(RuntimeKind::Winehq);
    }
    if path.components().any(|component| {
        matches!(
            normalized_component(component.as_os_str()).as_deref(),
            Some("custom") | Some("customwine")
        )
    }) {
        return Some(RuntimeKind::Custom);
    }
    None
}

fn canonical_wine_path(path: &Path) -> Option<PathBuf> {
    if !path.is_file() {
        return None;
    }
    let canonical = fs::canonicalize(path).ok()?;
    wine_has_64bit_kernel32(&canonical).then_some(canonical)
}

fn candidate_key(path: &Path) -> String {
    path.to_string_lossy().to_ascii_lowercase()
}

fn deduplicate_candidates(candidates: Vec<RuntimeCandidate>) -> Vec<RuntimeCandidate> {
    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|candidate| seen.insert(candidate_key(&candidate.path)))
        .collect()
}

fn resolve_detected_wine(kind: &RuntimeKind, runtimes: &[Runtime]) -> Option<Runtime> {
    runtimes
        .iter()
        .find(|runtime| runtime.kind == *kind && runtime.found)
        .cloned()
}

pub fn wine_root_from_bin(wine_bin: &Path) -> Option<PathBuf> {
    let resolved = fs::canonicalize(wine_bin).unwrap_or_else(|_| wine_bin.to_path_buf());
    let bin_dir = resolved.parent()?;
    bin_dir.parent().map(Path::to_path_buf)
}

fn wine_has_64bit_kernel32(wine_bin: &Path) -> bool {
    let Some(wine_root) = wine_root_from_bin(wine_bin) else {
        return false;
    };
    [wine_root.join("lib"), wine_root.join("lib64")]
        .into_iter()
        .map(|lib| lib.join("wine"))
        .any(|lib_wine| {
            let kernel32_pe = lib_wine.join("x86_64-windows").join("kernel32.dll");
            let kernel32_so = lib_wine.join("x86_64-unix").join("kernel32.so");
            let kernel32_unix_dll = lib_wine.join("x86_64-unix").join("kernel32.dll.so");
            kernel32_pe.is_file() || kernel32_so.is_file() || kernel32_unix_dll.is_file()
        })
}

pub fn backend_available(wine_bin: &Path, backend: &DxBackend) -> bool {
    let Some(root) = wine_root_from_bin(wine_bin) else {
        return false;
    };
    let path_text = root.to_string_lossy().to_ascii_lowercase();
    match backend {
        DxBackend::Wined3d => true,
        DxBackend::D3dmetal => {
            path_text.contains("game-porting-toolkit")
                || path_text.contains("crossover")
                || root.join("lib/external/libd3dshared.dylib").is_file()
                || Path::new("/System/Library/Frameworks/D3DMetal.framework").exists()
        }
        DxBackend::Dxvk => {
            root.join("lib/wine/dxvk").is_dir()
                || root.join("lib/dxvk").is_dir()
                || root.join("share/dxvk").is_dir()
        }
        DxBackend::Dxmt => {
            root.join("lib/wine/dxmt").is_dir()
                || root.join("lib/dxmt").is_dir()
                || path_text.contains("dxmt")
        }
    }
}

fn append_wine_binaries(paths: &mut Vec<PathBuf>, bin_dir: &Path) {
    for name in ["wine64", "wine"] {
        paths.push(bin_dir.join(name));
    }
}

fn scan_cellar(cellar: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    if let Ok(entries) = fs::read_dir(cellar) {
        for entry in entries.flatten() {
            let bin = entry.path().join("bin");
            append_wine_binaries(&mut found, &bin);
        }
    }
    found
}

fn gptk_candidates(home: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for bin_dir in [
        PathBuf::from("/usr/local/opt/game-porting-toolkit/bin"),
        PathBuf::from("/opt/homebrew/opt/game-porting-toolkit/bin"),
        PathBuf::from("/Applications/Game Porting Toolkit.app/Contents/Resources/wine/bin"),
        home.join("Library/Application Support/game-porting-toolkit/bin"),
    ] {
        append_wine_binaries(&mut paths, &bin_dir);
    }
    paths.extend(scan_cellar(Path::new(
        "/usr/local/Cellar/game-porting-toolkit",
    )));
    paths.extend(scan_cellar(Path::new(
        "/opt/homebrew/Cellar/game-porting-toolkit",
    )));
    paths
}

fn gcenx_candidates(home: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for bin_dir in [
        home.join(
            "Library/Application Support/Ponte/runtimes/wine-staging/11.16/Wine Staging.app/Contents/Resources/wine/bin",
        ),
        PathBuf::from("/Applications/Wine Staging.app/Contents/Resources/wine/bin"),
        PathBuf::from("/opt/homebrew/opt/wine@staging/bin"),
        PathBuf::from("/usr/local/opt/wine@staging/bin"),
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
        home.join("Library/Application Support/Heroic/tools/wine"),
        home.join("Library/Application Support/com.isaacmarovitz.Whisky/libraries/Wine/bin"),
    ] {
        append_wine_binaries(&mut paths, &bin_dir);
    }
    paths
}

fn winehq_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for bin_dir in [
        PathBuf::from("/Applications/Wine Stable.app/Contents/Resources/wine/bin"),
        PathBuf::from("/Applications/Wine Devel.app/Contents/Resources/wine/bin"),
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
    ] {
        append_wine_binaries(&mut paths, &bin_dir);
    }
    paths
}

fn winecx_candidates(home: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for bin_dir in [
        PathBuf::from("/opt/homebrew/opt/wine-crossover/bin"),
        PathBuf::from("/Applications/Wine CX.app/Contents/Resources/wine/bin"),
        PathBuf::from("/Applications/WineCX.app/Contents/Resources/wine/bin"),
        PathBuf::from("/Applications/Wine Crossover.app/Contents/Resources/wine/bin"),
        home.join("Library/Application Support/Heroic/tools/winecx"),
    ] {
        append_wine_binaries(&mut paths, &bin_dir);
    }
    paths
}

fn crossover_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    append_wine_binaries(
        &mut paths,
        Path::new("/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin"),
    );
    paths
}

fn is_unlabelled_staging_location(path: &Path) -> bool {
    let lower = path.to_string_lossy().to_ascii_lowercase();
    lower.contains("/heroic/tools/wine/")
        || lower.contains("/whisky/libraries/wine/")
        || lower.contains("/wine-staging/bin/wine")
}

fn kind_for_candidate(path: &Path, hint: RuntimeKind) -> Option<RuntimeKind> {
    if let Some(kind) = classify_runtime_path(path) {
        return Some(kind);
    }

    match hint {
        // Heroic and Whisky can hide the distribution name behind a generic
        // `Wine` directory after symlink resolution. Their candidate roots are
        // still specific enough to identify the Gcenx staging family.
        RuntimeKind::GcenxStaging if is_unlabelled_staging_location(path) => {
            Some(RuntimeKind::GcenxStaging)
        }
        // An explicitly configured path may point to a private Wine build.
        RuntimeKind::Custom => Some(RuntimeKind::Custom),
        // A generic path is not enough evidence for GPTK, WineCX, WineHQ or
        // CrossOver. Their canonical path must carry a product marker.
        RuntimeKind::Gptk
        | RuntimeKind::Winehq
        | RuntimeKind::Winecx
        | RuntimeKind::Crossover
        | RuntimeKind::GcenxStaging => None,
    }
}

fn runtime_candidates(home: &Path, custom_wine_path: &str) -> Vec<RuntimeCandidate> {
    let groups = [
        (RuntimeKind::Gptk, gptk_candidates(home)),
        (RuntimeKind::GcenxStaging, gcenx_candidates(home)),
        (RuntimeKind::Winehq, winehq_candidates()),
        (RuntimeKind::Winecx, winecx_candidates(home)),
        (RuntimeKind::Crossover, crossover_candidates()),
    ];
    let mut candidates = Vec::new();

    for (hint, paths) in groups {
        for path in paths {
            let Some(path) = canonical_wine_path(&path) else {
                continue;
            };
            let Some(kind) = kind_for_candidate(&path, hint) else {
                continue;
            };
            candidates.push(RuntimeCandidate { kind, path });
        }
    }

    let custom_wine_path = custom_wine_path.trim();
    if !custom_wine_path.is_empty() {
        let path = expand_home(custom_wine_path);
        if let Some(path) = canonical_wine_path(&path) {
            if let Some(kind) = kind_for_candidate(&path, RuntimeKind::Custom) {
                candidates.push(RuntimeCandidate { kind, path });
            }
        }
    }

    deduplicate_candidates(candidates)
}

pub fn detect_runtimes() -> Vec<Runtime> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let settings = read_settings().unwrap_or_default();
    let candidates = runtime_candidates(&home, &settings.custom_wine_path);

    CATALOG
        .iter()
        .map(|entry| {
            let wine_path = candidates
                .iter()
                .find(|candidate| candidate.kind == entry.kind)
                .map(|candidate| &candidate.path);

            Runtime {
                kind: entry.kind,
                label: entry.label.to_string(),
                wine64_path: wine_path
                    .as_ref()
                    .map(|path| path.to_string_lossy().to_string())
                    .unwrap_or_default(),
                found: wine_path.is_some(),
                supported_backends: entry.backends.to_vec(),
                install_url: entry.install_url.map(str::to_string),
            }
        })
        .collect()
}

pub fn resolve_wine(kind: &RuntimeKind) -> Option<Runtime> {
    resolve_detected_wine(kind, &detect_runtimes())
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_wine_path, classify_runtime_path, deduplicate_candidates, gcenx_candidates,
        kind_for_candidate, normalized_component, resolve_detected_wine, RuntimeCandidate,
    };
    use crate::types::{Runtime, RuntimeKind};
    use std::path::{Path, PathBuf};

    #[test]
    fn classifies_known_runtime_paths() {
        let cases = [
            (
                "/Applications/Wine Staging.app/Contents/Resources/wine/bin/wine",
                RuntimeKind::GcenxStaging,
            ),
            (
                "/Users/test/Library/Application Support/Ponte/runtimes/wine-staging/11.16/Wine Staging.app/Contents/Resources/wine/bin/wine",
                RuntimeKind::GcenxStaging,
            ),
            (
                "/Applications/Wine Stable.app/Contents/Resources/wine/bin/wine64",
                RuntimeKind::Winehq,
            ),
            (
                "/opt/homebrew/opt/game-porting-toolkit/bin/wine",
                RuntimeKind::Gptk,
            ),
            (
                "/opt/homebrew/opt/wine-crossover/bin/wine64",
                RuntimeKind::Winecx,
            ),
            (
                "/Applications/Wine CX.app/Contents/Resources/wine/bin/wine",
                RuntimeKind::Winecx,
            ),
            (
                "/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine",
                RuntimeKind::Crossover,
            ),
            (
                "/Users/test/Library/Application Support/Ponte/runtimes/custom/bin/wine",
                RuntimeKind::Custom,
            ),
        ];

        for (path, expected) in cases {
            assert_eq!(classify_runtime_path(Path::new(path)), Some(expected));
        }
        assert_eq!(
            classify_runtime_path(Path::new("/Users/test/Tools/my-wine/bin/wine")),
            None
        );
        assert_eq!(
            kind_for_candidate(
                Path::new("/opt/homebrew/bin/wine"),
                RuntimeKind::GcenxStaging
            ),
            None
        );
        assert_eq!(
            kind_for_candidate(Path::new("/opt/homebrew/bin/wine"), RuntimeKind::Winehq),
            None
        );
    }

    #[test]
    fn classifies_crossover_before_winecx_when_both_names_are_present() {
        let path =
            Path::new("/Applications/CrossOver.app/Contents/SharedSupport/wine-crossover/bin/wine");
        assert_eq!(classify_runtime_path(path), Some(RuntimeKind::Crossover));
    }

    #[test]
    fn normalizes_app_names_without_relying_on_case_or_punctuation() {
        assert_eq!(
            normalized_component(std::ffi::OsStr::new("Wine Staging.app")),
            Some("winestagingapp".to_string())
        );
        assert_eq!(
            normalized_component(std::ffi::OsStr::new("game-porting-toolkit")),
            Some("gameportingtoolkit".to_string())
        );
    }

    #[test]
    fn includes_managed_staging_and_both_wine_entrypoints() {
        let candidates = gcenx_candidates(Path::new("/Users/test"));
        let managed = "/Users/test/Library/Application Support/Ponte/runtimes/wine-staging/11.16/Wine Staging.app/Contents/Resources/wine/bin";
        assert!(candidates.contains(&PathBuf::from(format!("{managed}/wine"))));
        assert!(candidates.contains(&PathBuf::from(format!("{managed}/wine64"))));
    }

    #[test]
    fn deduplicates_canonical_paths_case_insensitively() {
        let path = PathBuf::from("/Applications/Wine Staging.app/Contents/Resources/wine/bin/wine");
        let duplicate =
            PathBuf::from("/applications/wine staging.app/contents/resources/wine/bin/WINE");
        let candidates = deduplicate_candidates(vec![
            RuntimeCandidate {
                kind: RuntimeKind::GcenxStaging,
                path: path.clone(),
            },
            RuntimeCandidate {
                kind: RuntimeKind::Winehq,
                path: duplicate,
            },
        ]);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].kind, RuntimeKind::GcenxStaging);
        assert_eq!(candidates[0].path, path);
    }

    #[test]
    fn explicit_runtime_resolution_does_not_fallback() {
        let stable = Runtime {
            kind: RuntimeKind::Winehq,
            label: "WineHQ Stable".to_string(),
            wine64_path: "/Applications/Wine Stable.app/Contents/Resources/wine/bin/wine"
                .to_string(),
            found: true,
            supported_backends: vec![],
            install_url: None,
        };

        assert!(resolve_detected_wine(&RuntimeKind::Gptk, &[stable.clone()]).is_none());
        assert_eq!(
            resolve_detected_wine(&RuntimeKind::Winehq, &[stable]).map(|runtime| runtime.kind),
            Some(RuntimeKind::Winehq)
        );
    }

    #[test]
    fn accepts_wine_and_wine64_when_x64_kernel32_is_present() {
        use std::fs::{self, File};
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ponte-runtime-kernel32-{nonce}"));
        let bin = root.join("bin");
        let kernel32 = root.join("lib/wine/x86_64-windows/kernel32.dll");
        fs::create_dir_all(&bin).expect("create wine bin");
        fs::create_dir_all(kernel32.parent().expect("kernel32 parent")).expect("create wine lib");
        File::create(&kernel32).expect("create x64 kernel32");

        let wine = bin.join("wine");
        let wine64 = bin.join("wine64");
        File::create(&wine).expect("create wine launcher");
        File::create(&wine64).expect("create wine64 launcher");

        assert!(canonical_wine_path(&wine).is_some());
        assert!(canonical_wine_path(&wine64).is_some());

        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[cfg(unix)]
    #[test]
    fn deduplicates_symlinked_aliases_after_canonicalization() {
        use std::fs::{self, File};
        use std::os::unix::fs::symlink;
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ponte-runtime-dedupe-{nonce}"));
        let bin = root.join("Wine Staging.app/Contents/Resources/wine/bin");
        let kernel32 = root
            .join("Wine Staging.app/Contents/Resources/wine/lib/wine/x86_64-windows/kernel32.dll");
        fs::create_dir_all(&bin).expect("create wine bin");
        fs::create_dir_all(kernel32.parent().expect("kernel32 parent")).expect("create wine lib");
        File::create(&kernel32).expect("create x64 kernel32");

        let wine = bin.join("wine");
        File::create(&wine).expect("create wine launcher");
        let alias = root.join("alias/bin/wine");
        fs::create_dir_all(alias.parent().expect("alias parent")).expect("create alias dir");
        symlink(&wine, &alias).expect("create wine alias");

        let canonical = canonical_wine_path(&wine).expect("canonical wine");
        let canonical_alias = canonical_wine_path(&alias).expect("canonical alias");
        assert_eq!(canonical, canonical_alias);

        let candidates = deduplicate_candidates(vec![
            RuntimeCandidate {
                kind: RuntimeKind::GcenxStaging,
                path: canonical,
            },
            RuntimeCandidate {
                kind: RuntimeKind::Winehq,
                path: canonical_alias,
            },
        ]);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].kind, RuntimeKind::GcenxStaging);

        fs::remove_dir_all(root).expect("remove test runtime");
    }
}
