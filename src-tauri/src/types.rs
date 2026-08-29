use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeKind {
    Gptk,
    GcenxStaging,
    Winecx,
    Crossover,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DxBackend {
    D3dmetal,
    Dxmt,
    Dxvk,
    Wined3d,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PrefixStatus {
    Missing,
    Ready,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LaunchState {
    Idle,
    Starting,
    Running,
    Stopping,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Runtime {
    pub kind: RuntimeKind,
    pub label: String,
    pub wine64_path: String,
    pub found: bool,
    pub supported_backends: Vec<DxBackend>,
    pub install_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub name: String,
    pub exe_path: String,
    pub args: String,
    pub runtime_kind: RuntimeKind,
    pub dx_backend: DxBackend,
    pub prefix_path: String,
    pub prefix_status: PrefixStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix_error: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub custom_wine_path: String,
    pub prefixes_dir: String,
    pub metal_hud: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameLog {
    pub game_id: String,
    pub lines: Vec<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddGameInput {
    pub name: String,
    pub exe_path: String,
    pub args: String,
    pub runtime_kind: RuntimeKind,
    pub dx_backend: DxBackend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryFile {
    pub version: u32,
    pub games: Vec<Game>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            custom_wine_path: String::new(),
            prefixes_dir: String::from("~/Library/Application Support/Ponte/prefixes"),
            metal_hud: false,
        }
    }
}
