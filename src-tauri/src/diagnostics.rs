use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::runtimes::{backend_available, detect_runtimes};
use crate::setup::recommended_stack_for_engine;
use crate::storage::ensure_ponte_dir;
use crate::types::{
    CheckStatus, DiagnosticCheck, DxBackend, EngineGuess, RuntimeKind, SystemDiagnostics,
};

fn now_iso() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn check(
    id: &str,
    label: &str,
    status: CheckStatus,
    detail: impl Into<String>,
    remediation: Option<&str>,
) -> DiagnosticCheck {
    DiagnosticCheck {
        id: id.to_string(),
        label: label.to_string(),
        status,
        detail: detail.into(),
        remediation: remediation.map(str::to_string),
    }
}

fn writable_data_check() -> DiagnosticCheck {
    let Ok(dir) = ensure_ponte_dir() else {
        return check(
            "data-writable",
            "Dados do Ponte",
            CheckStatus::Fail,
            "Não foi possível preparar Application Support/Ponte.",
            Some("Verifique as permissões do disco e tente novamente."),
        );
    };
    let marker = dir.join(".diagnostic-write-test");
    match fs::write(&marker, b"ok") {
        Ok(()) => {
            let _ = fs::remove_file(marker);
            check(
                "data-writable",
                "Dados do Ponte",
                CheckStatus::Pass,
                format!("Pasta gravável: {}", dir.display()),
                None,
            )
        }
        Err(error) => check(
            "data-writable",
            "Dados do Ponte",
            CheckStatus::Fail,
            format!("A pasta de dados não é gravável: {error}"),
            Some("Escolha uma pasta de dados com permissão de escrita."),
        ),
    }
}

fn rosetta_check() -> DiagnosticCheck {
    if cfg!(target_os = "macos") && std::env::consts::ARCH == "aarch64" {
        match Command::new("arch")
            .args(["-x86_64", "/usr/bin/true"])
            .status()
        {
            Ok(status) if status.success() => check(
                "rosetta",
                "Rosetta 2",
                CheckStatus::Pass,
                "Rosetta 2 consegue executar binários x86_64.",
                None,
            ),
            Ok(status) => check(
                "rosetta",
                "Rosetta 2",
                CheckStatus::Fail,
                format!("Rosetta 2 recusou o teste ({status})."),
                Some("Instale Rosetta 2 com: softwareupdate --install-rosetta."),
            ),
            Err(error) => check(
                "rosetta",
                "Rosetta 2",
                CheckStatus::Fail,
                format!("Não foi possível testar Rosetta 2: {error}"),
                Some("Instale Rosetta 2 e reabra o Ponte."),
            ),
        }
    } else {
        check(
            "rosetta",
            "Rosetta 2",
            CheckStatus::Unknown,
            format!("Arquitetura nativa: {}.", std::env::consts::ARCH),
            None,
        )
    }
}

fn gst_check() -> DiagnosticCheck {
    let candidates = [
        "/opt/homebrew/bin/gst-inspect-1.0",
        "/usr/local/bin/gst-inspect-1.0",
    ];
    if let Some(path) = candidates.iter().find(|path| Path::new(path).is_file()) {
        return check(
            "gstreamer",
            "Áudio GStreamer",
            CheckStatus::Pass,
            format!("Encontrado em {path}; áudio pode ser inicializado pelo Wine."),
            None,
        );
    }
    check(
        "gstreamer",
        "Áudio GStreamer",
        CheckStatus::Warning,
        "GStreamer não foi encontrado nos caminhos padrão.",
        Some("Instale o runtime GStreamer se um jogo precisar de áudio."),
    )
}

pub fn collect() -> SystemDiagnostics {
    let runtimes = detect_runtimes();
    let (preferred_runtime, preferred_backend) =
        recommended_stack_for_engine(&EngineGuess::Unknown);
    let selected_runtime = runtimes
        .iter()
        .find(|runtime| runtime.kind == preferred_runtime && runtime.found)
        .or_else(|| runtimes.iter().find(|runtime| runtime.found))
        .map(|runtime| runtime.kind);
    let selected_backend = selected_runtime.map(|runtime| {
        if runtime == preferred_runtime {
            preferred_backend
        } else {
            match runtime {
                RuntimeKind::Gptk | RuntimeKind::Crossover => DxBackend::D3dmetal,
                RuntimeKind::GcenxStaging | RuntimeKind::Winecx | RuntimeKind::Winehq => {
                    DxBackend::Dxvk
                }
                RuntimeKind::Custom => DxBackend::Wined3d,
            }
        }
    });

    let mut checks = vec![
        check(
            "host-architecture",
            "Arquitetura do Mac",
            if cfg!(target_arch = "aarch64") {
                CheckStatus::Pass
            } else {
                CheckStatus::Warning
            },
            format!("Processo nativo em {}.", std::env::consts::ARCH),
            None,
        ),
        rosetta_check(),
        writable_data_check(),
        gst_check(),
    ];

    checks.push(match selected_runtime {
        Some(kind) => check(
            "runtime",
            "Runtime Windows",
            CheckStatus::Pass,
            format!(
                "{} disponível.",
                runtimes
                    .iter()
                    .find(|item| item.kind == kind)
                    .map(|item| item.label.as_str())
                    .unwrap_or("Wine")
            ),
            None,
        ),
        None => check(
            "runtime",
            "Runtime Windows",
            CheckStatus::Fail,
            "Nenhum Wine x64 validado foi encontrado.",
            Some("Instale o Wine Staging gerenciado pelo Ponte."),
        ),
    });

    if let Some(kind) = selected_runtime {
        if let Some(runtime) = runtimes.iter().find(|item| item.kind == kind) {
            let backend = selected_backend.unwrap_or(DxBackend::Wined3d);
            checks.push(check(
                "graphics-backend",
                "Backend gráfico",
                if backend == DxBackend::Wined3d
                    || !backend_available(Path::new(&runtime.wine64_path), &backend)
                {
                    CheckStatus::Warning
                } else {
                    CheckStatus::Pass
                },
                format!(
                    "{} selecionado para {}.",
                    format_backend(backend),
                    runtime.label
                ),
                Some("O preflight do jogo valida o prefixo e pode instalar DXVK."),
            ));
        }
    }

    let ready = checks.iter().all(|item| item.status != CheckStatus::Fail);
    SystemDiagnostics {
        checked_at: now_iso(),
        ready,
        checks,
        selected_runtime,
        selected_backend,
    }
}

fn format_backend(backend: DxBackend) -> &'static str {
    match backend {
        DxBackend::D3dmetal => "D3DMetal",
        DxBackend::Dxmt => "DXMT",
        DxBackend::Dxvk => "DXVK",
        DxBackend::Wined3d => "WineD3D",
    }
}

#[cfg(test)]
mod tests {
    use super::format_backend;
    use crate::types::DxBackend;

    #[test]
    fn labels_graphics_backends() {
        assert_eq!(format_backend(DxBackend::Dxvk), "DXVK");
    }
}
