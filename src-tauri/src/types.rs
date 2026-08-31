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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warning,
    Fail,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticCheck {
    pub id: String,
    pub label: String,
    pub status: CheckStatus,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemDiagnostics {
    pub checked_at: String,
    pub ready: bool,
    pub checks: Vec<DiagnosticCheck>,
    pub selected_runtime: Option<RuntimeKind>,
    pub selected_backend: Option<DxBackend>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PreflightStatus {
    Ready,
    Warning,
    Blocked,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightCheck {
    pub id: String,
    pub label: String,
    pub status: CheckStatus,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GamePreflight {
    pub game_id: String,
    pub executable: String,
    pub fingerprint: String,
    pub engine_guess: EngineGuess,
    pub status: PreflightStatus,
    pub checks: Vec<PreflightCheck>,
    pub recommended_runtime: RuntimeKind,
    pub recommended_backend: DxBackend,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LaunchAttemptStatus {
    Queued,
    Preflighting,
    Preparing,
    Probing,
    Starting,
    Running,
    Stopping,
    Succeeded,
    Failed,
    Blocked,
    Interrupted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchFailure {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchAttempt {
    pub id: String,
    pub game_id: String,
    pub status: LaunchAttemptStatus,
    pub started_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    pub executable: String,
    pub working_dir: String,
    pub args: Vec<String>,
    pub runtime_kind: RuntimeKind,
    pub runtime_path: String,
    pub dx_backend: DxBackend,
    pub prefix_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preflight_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<LaunchFailure>,
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
