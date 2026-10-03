use std::fs;
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::runtimes::{backend_available, resolve_wine};
use crate::setup::{
    args_force_dx11, game_requires_d3d12, guess_engine, prefix_has_dxvk, recommended_stack_for_game,
};
use crate::storage::expand_home;
use crate::types::{
    CheckStatus, DxBackend, Game, GamePreflight, PreflightCheck, PreflightStatus, Runtime,
    RuntimeKind,
};

fn check(
    id: &str,
    label: &str,
    status: CheckStatus,
    detail: impl Into<String>,
    remediation: Option<&str>,
) -> PreflightCheck {
    PreflightCheck {
        id: id.to_string(),
        label: label.to_string(),
        status,
        detail: detail.into(),
        remediation: remediation.map(str::to_string),
    }
}

fn digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn machine(bytes: &[u8]) -> Option<u16> {
    if bytes.len() < 64 || &bytes[..2] != b"MZ" {
        return None;
    }
    let pe_offset = u32::from_le_bytes(bytes[0x3c..0x40].try_into().ok()?) as usize;
    if pe_offset.checked_add(6)? > bytes.len() || &bytes[pe_offset..pe_offset + 4] != b"PE\0\0" {
        return None;
    }
    Some(u16::from_le_bytes(
        bytes[pe_offset + 4..pe_offset + 6].try_into().ok()?,
    ))
}

fn architecture_label(machine: Option<u16>) -> &'static str {
    match machine {
        Some(0x8664) => "x86_64",
        Some(0x14c) => "x86",
        Some(0xaa64) => "arm64",
        Some(_) => "Windows desconhecido",
        None => "não é PE Windows",
    }
}

fn has_import(bytes: &[u8], name: &[u8]) -> bool {
    bytes
        .windows(name.len())
        .any(|window| window.eq_ignore_ascii_case(name))
}

fn file_bytes(path: &Path) -> Result<Vec<u8>, String> {
    let mut file =
        fs::File::open(path).map_err(|error| format!("Não foi possível abrir: {error}"))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| format!("Não foi possível ler: {error}"))?;
    Ok(bytes)
}

fn status_for(checks: &[PreflightCheck]) -> PreflightStatus {
    if checks.iter().any(|item| item.status == CheckStatus::Fail) {
        PreflightStatus::Blocked
    } else if checks
        .iter()
        .any(|item| item.status == CheckStatus::Warning)
    {
        PreflightStatus::Warning
    } else if checks
        .iter()
        .all(|item| item.status == CheckStatus::Unknown)
    {
        PreflightStatus::Unknown
    } else {
        PreflightStatus::Ready
    }
}

fn validated_d3d12_profile(game: &Game, runtime: Option<&Runtime>) -> bool {
    game.dx_backend == DxBackend::D3dmetal
        && runtime.is_some_and(|runtime| {
            matches!(runtime.kind, RuntimeKind::Gptk | RuntimeKind::Crossover)
                && backend_available(Path::new(&runtime.wine64_path), &DxBackend::D3dmetal)
        })
}

pub fn inspect(game: &Game) -> GamePreflight {
    let executable = expand_home(&game.exe_path);
    let bytes = file_bytes(&executable).unwrap_or_default();
    let machine = machine(&bytes);
    let engine = guess_engine(&executable);
    let (recommended_runtime, recommended_backend) = recommended_stack_for_game(game);
    let mut checks = Vec::new();

    if bytes.is_empty() {
        checks.push(check(
            "executable",
            "Executável",
            CheckStatus::Fail,
            format!("Ficheiro não encontrado ou vazio: {}", executable.display()),
            Some("Adicione novamente a pasta que contém o jogo."),
        ));
    } else if machine.is_none() {
        checks.push(check(
            "executable",
            "Executável",
            CheckStatus::Fail,
            "O ficheiro não tem cabeçalho PE Windows válido.",
            Some("Selecione o .exe principal do jogo, não um instalador ou utilitário."),
        ));
    } else {
        checks.push(check(
            "executable",
            "Executável",
            CheckStatus::Pass,
            format!("PE Windows válido: {}.", architecture_label(machine)),
            None,
        ));
    }

    let helper = executable
        .file_name()
        .and_then(|name| name.to_str())
        .map(crate::discover::is_helper_exe)
        .unwrap_or(false);
    if helper {
        checks.push(check(
            "entrypoint",
            "Ponto de entrada",
            CheckStatus::Fail,
            "Este .exe parece ser um verificador, instalador ou redistributable.",
            Some("Adicione a pasta do jogo para o Ponte escolher o executável principal."),
        ));
    } else {
        checks.push(check(
            "entrypoint",
            "Ponto de entrada",
            CheckStatus::Pass,
            "O nome do executável não corresponde a um helper conhecido.",
            None,
        ));
    }

    match machine {
        Some(0x8664) | Some(0xaa64) => checks.push(check(
            "architecture",
            "Arquitetura do jogo",
            CheckStatus::Pass,
            format!(
                "{} compatível com o caminho Windows x64/ARM64.",
                architecture_label(machine)
            ),
            None,
        )),
        Some(0x14c) => checks.push(check(
            "architecture",
            "Arquitetura do jogo",
            CheckStatus::Warning,
            "O jogo é x86 (32-bit); pode exigir uma configuração adicional no prefixo.",
            Some("Use um prefixo dedicado e valide o jogo real antes de publicar o perfil."),
        )),
        Some(_) => checks.push(check(
            "architecture",
            "Arquitetura do jogo",
            CheckStatus::Fail,
            format!(
                "Arquitetura PE não suportada: {}.",
                architecture_label(machine)
            ),
            Some("Use uma versão Windows x64 ou x86 conhecida."),
        )),
        None => {}
    }

    let runtime = resolve_wine(&game.runtime_kind);
    checks.push(match runtime {
        Some(ref runtime) => check(
            "runtime",
            "Runtime selecionado",
            CheckStatus::Pass,
            if runtime.kind == RuntimeKind::Gptk {
                format!("GPTK + D3DMetal validados em {}.", runtime.wine64_path)
            } else {
                format!("{} em {}.", runtime.label, runtime.wine64_path)
            },
            None,
        ),
        None => check(
            "runtime",
            "Runtime selecionado",
            CheckStatus::Warning,
            "O runtime configurado ainda não está disponível; a preparação pode instalá-lo.",
            Some("Execute Preparar jogo para instalar o Wine Staging gerenciado."),
        ),
    });

    let imports_d3d12 = has_import(&bytes, b"d3d12.dll");
    if imports_d3d12 && args_force_dx11(&game.args) {
        checks.push(check(
            "graphics-api",
            "API gráfica",
            CheckStatus::Warning,
            "O executável importa D3D12, mas o perfil força D3D11. O Ponte permitirá a tentativa pelo backend atual.",
            Some("Se o jogo ignorar -dx11, remova o argumento e use GPTK/CrossOver com D3DMetal validado."),
        ));
    } else if game_requires_d3d12(game) && validated_d3d12_profile(game, runtime.as_ref()) {
        checks.push(check(
            "graphics-api",
            "API gráfica",
            CheckStatus::Pass,
            "D3D12 coberto por um perfil GPTK/CrossOver com D3DMetal validado.",
            None,
        ));
    } else if game_requires_d3d12(game) {
        checks.push(check(
            "graphics-api",
            "API gráfica",
            CheckStatus::Fail,
            "O executável importa D3D12; o perfil atual não oferece um caminho D3D12 seguro.",
            Some("Use um perfil GPTK/CrossOver validado ou force uma versão DX11 compatível."),
        ));
    } else if has_import(&bytes, b"d3d11.dll") {
        checks.push(check(
            "graphics-api",
            "API gráfica",
            CheckStatus::Pass,
            "O executável importa D3D11.",
            None,
        ));
    } else {
        checks.push(check(
            "graphics-api",
            "API gráfica",
            CheckStatus::Unknown,
            "Não foi possível inferir a API gráfica apenas pelos imports.",
            None,
        ));
    }

    let prefix = expand_home(&game.prefix_path);
    checks.push(
        if game.prefix_status == crate::types::PrefixStatus::Ready
            && prefix.join("system.reg").is_file()
        {
            check(
                "prefix",
                "Prefixo",
                CheckStatus::Pass,
                format!("Prefixo pronto em {}.", prefix.display()),
                None,
            )
        } else {
            check(
                "prefix",
                "Prefixo",
                CheckStatus::Warning,
                "O prefixo ainda precisa ser criado ou reparado.",
                Some("Use Preparar jogo; o Ponte cria um prefixo isolado."),
            )
        },
    );
    checks.push(if crate::launch::vcrun_is_installed(&prefix) {
        check(
            "vcrun",
            "Visual C++",
            CheckStatus::Pass,
            "Visual C++ nativo já está instalado no prefixo.",
            None,
        )
    } else {
        check(
            "vcrun",
            "Visual C++",
            CheckStatus::Warning,
            "Visual C++ nativo ainda não foi confirmado neste prefixo.",
            Some("Use Preparar jogo para instalar ou reutilizar a cache oficial."),
        )
    });

    if game.dx_backend == DxBackend::Dxvk
        && game.prefix_status == crate::types::PrefixStatus::Ready
        && !prefix_has_dxvk(&prefix)
    {
        checks.push(check(
            "graphics-runtime",
            "Runtime DirectX",
            CheckStatus::Warning,
            "DXVK ainda não foi marcado neste prefixo.",
            Some("Prepare o jogo antes de iniciar."),
        ));
    }

    GamePreflight {
        game_id: game.id.clone(),
        executable: executable.to_string_lossy().into_owned(),
        fingerprint: digest(&bytes),
        engine_guess: engine,
        status: status_for(&checks),
        checks,
        recommended_runtime,
        recommended_backend,
    }
}

#[cfg(test)]
mod tests {
    use super::{architecture_label, machine};

    #[test]
    fn reads_pe_x64_machine() {
        let mut bytes = vec![0_u8; 80];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[0x3c..0x40].copy_from_slice(&64_u32.to_le_bytes());
        bytes[64..68].copy_from_slice(b"PE\0\0");
        bytes[68..70].copy_from_slice(&0x8664_u16.to_le_bytes());
        assert_eq!(machine(&bytes), Some(0x8664));
        assert_eq!(architecture_label(machine(&bytes)), "x86_64");
    }
}
