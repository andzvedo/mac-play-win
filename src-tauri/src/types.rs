use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeKind {
    Gptk,
    GcenxStaging,
    Winehq,
    Winecx,
    Crossover,
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
    #[serde(default)]
    pub install_dir: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EngineGuess {
    Unity,
    Unreal,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SetupItemStatus {
    Ready,
    Missing,
    Installable,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupItem {
    pub id: String,
    pub label: String,
    pub status: SetupItemStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSetupPlan {
    pub game_id: String,
    pub engine_guess: EngineGuess,
    pub recommended_runtime: RuntimeKind,
    pub recommended_backend: DxBackend,
    pub items: Vec<SetupItem>,
    pub can_auto_fix: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDiscovery {
    pub folder_path: String,
    pub exe_path: String,
    pub suggested_name: String,
    pub engine_guess: EngineGuess,
    pub recommended_runtime: RuntimeKind,
    pub recommended_backend: DxBackend,
    #[serde(default)]
    pub setup_items: Vec<SetupItem>,
    #[serde(default)]
    pub can_auto_fix: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddGameInput {
    pub name: String,
    pub folder_path: String,
    pub args: String,
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
