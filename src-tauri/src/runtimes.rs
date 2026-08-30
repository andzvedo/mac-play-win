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

fn first_existing(paths: &[PathBuf]) -> Option<PathBuf> {
    paths
        .iter()
        .find(|path| path.is_file() && wine_has_64bit_kernel32(path))
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
    let lib_wine = wine_root.join("lib").join("wine");
    let kernel32_pe = lib_wine.join("x86_64-windows").join("kernel32.dll");
    let kernel32_so = lib_wine.join("x86_64-unix").join("kernel32.so");
    kernel32_pe.is_file() || kernel32_so.is_file()
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

fn scan_cellar(cellar: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    if let Ok(entries) = fs::read_dir(cellar) {
        for entry in entries.flatten() {
            let bin = entry.path().join("bin");
            for name in ["wine64", "wine"] {
                let wine = bin.join(name);
                if wine.is_file() {
                    found.push(wine);
                }
            }
        }
    }
    found
}

fn gptk_candidates(home: &Path) -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("/usr/local/opt/game-porting-toolkit/bin/wine64"),
        PathBuf::from("/opt/homebrew/opt/game-porting-toolkit/bin/wine64"),
        home.join("Library/Application Support/game-porting-toolkit/bin/wine64"),
    ];
    paths.extend(scan_cellar(Path::new(
        "/usr/local/Cellar/game-porting-toolkit",
    )));
    paths.extend(scan_cellar(Path::new(
        "/opt/homebrew/Cellar/game-porting-toolkit",
    )));
    paths
}

fn gcenx_candidates(home: &Path) -> Vec<PathBuf> {
    vec![
        PathBuf::from("/opt/homebrew/bin/wine64"),
        PathBuf::from("/usr/local/bin/wine64"),
        home.join("Library/Application Support/Heroic/tools/wine/wine64"),
        home.join(
            "Library/Application Support/com.isaacmarovitz.Whisky/libraries/Wine/bin/wine64",
        ),
    ]
}

fn winehq_candidates() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/Applications/Wine Stable.app/Contents/Resources/wine/bin/wine"),
        PathBuf::from("/Applications/Wine Staging.app/Contents/Resources/wine/bin/wine"),
        PathBuf::from("/Applications/Wine Devel.app/Contents/Resources/wine/bin/wine"),
        PathBuf::from("/opt/homebrew/bin/wine"),
        PathBuf::from("/usr/local/bin/wine"),
    ]
}

fn winecx_candidates(home: &Path) -> Vec<PathBuf> {
    vec![
        PathBuf::from("/opt/homebrew/opt/wine-crossover/bin/wine64"),
        home.join("Library/Application Support/Heroic/tools/winecx/wine64"),
    ]
}

fn crossover_candidates() -> Vec<PathBuf> {
    vec![
        PathBuf::from(
            "/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine",
        ),
        PathBuf::from(
            "/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine64",
        ),
    ]
}

pub fn detect_runtimes() -> Vec<Runtime> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let settings = read_settings().unwrap_or_default();

    CATALOG
        .iter()
        .map(|entry| {
            let wine_path = match entry.kind {
                RuntimeKind::Gptk => first_existing(&gptk_candidates(&home)),
                RuntimeKind::GcenxStaging => first_existing(&gcenx_candidates(&home)),
                RuntimeKind::Winehq => first_existing(&winehq_candidates()),
                RuntimeKind::Winecx => first_existing(&winecx_candidates(&home)),
                RuntimeKind::Crossover => first_existing(&crossover_candidates()),
                RuntimeKind::Custom => {
                    let custom = settings.custom_wine_path.trim();
                    if custom.is_empty() {
                        None
                    } else {
                        let path = expand_home(custom);
                        (path.is_file() && wine_has_64bit_kernel32(&path)).then_some(path)
                    }
                }
            };

            Runtime {
                kind: entry.kind.clone(),
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
    let runtimes = detect_runtimes();
    if let Some(runtime) = runtimes
        .iter()
        .find(|item| item.kind == *kind && item.found)
    {
        return Some(runtime.clone());
    }
    runtimes.into_iter().find(|item| item.found)
}
