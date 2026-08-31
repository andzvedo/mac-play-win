use crate::types::DxBackend;

const VCRUN_OVERRIDE: &str =
    "vcruntime140,vcruntime140_1,msvcp140,msvcp140_1,msvcp140_2,concrt140,vccorlib140=n,b";

fn with_vcrun(overrides: &str) -> String {
    format!("{overrides};{VCRUN_OVERRIDE}")
}

pub fn env_for_backend(backend: &DxBackend, metal_hud: bool) -> Vec<(String, String)> {
    let mut env = vec![(
        "MTL_HUD_ENABLED".to_string(),
        if metal_hud {
            "1".to_string()
        } else {
            "0".to_string()
        },
    )];

    match backend {
        DxBackend::D3dmetal => {
            env.push((
                "WINEDLLOVERRIDES".to_string(),
                with_vcrun("d3d11,d3d12,dxgi=n,b"),
            ));
            env.push(("D3DMETAL_ENABLE".to_string(), "1".to_string()));
        }
        DxBackend::Dxmt => {
            env.push((
                "WINEDLLOVERRIDES".to_string(),
                with_vcrun("d3d10,d3d11,dxgi=n,b"),
            ));
            env.push(("DXMT_CONFIG".to_string(), "dxmt".to_string()));
        }
        DxBackend::Dxvk => {
            env.push((
                "WINEDLLOVERRIDES".to_string(),
                with_vcrun("d3d10core,d3d11,dxgi=n,b"),
            ));
            env.push((
                "DXVK_HUD".to_string(),
                if metal_hud {
                    "1".to_string()
                } else {
                    "0".to_string()
                },
            ));
        }
        DxBackend::Wined3d => {
            env.push((
                "WINEDLLOVERRIDES".to_string(),
                with_vcrun("d3d11,d3d12,dxgi=b"),
            ));
        }
    }

    env
}

#[cfg(test)]
mod tests {
    use super::env_for_backend;
    use crate::types::DxBackend;

    #[test]
    fn d3dmetal_sets_overrides() {
        let env = env_for_backend(&DxBackend::D3dmetal, true);
        let hud = env
            .iter()
            .find(|(key, _)| key == "MTL_HUD_ENABLED")
            .unwrap();
        assert_eq!(hud.1, "1");
        let overrides = env
            .iter()
            .find(|(key, _)| key == "WINEDLLOVERRIDES")
            .unwrap();
        assert!(overrides.1.contains("vcruntime140"));
    }

    #[test]
    fn wined3d_appends_vcrun_native() {
        let env = env_for_backend(&DxBackend::Wined3d, false);
        let overrides = env
            .iter()
            .find(|(key, _)| key == "WINEDLLOVERRIDES")
            .unwrap();
        assert_eq!(
            overrides.1,
            "d3d11,d3d12,dxgi=b;vcruntime140,vcruntime140_1,msvcp140,msvcp140_1,msvcp140_2,concrt140,vccorlib140=n,b"
        );
    }
}
