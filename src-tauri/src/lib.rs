mod attempts;
mod diagnostics;
mod discover;
mod env_backend;
mod error;
mod launch;
mod paths;
mod preflight;
mod runtimes;
mod setup;
mod storage;
mod types;

use std::sync::Arc;

use launch::LaunchRegistry;
use tauri::State;
use tauri_plugin_dialog::{DialogExt, FilePath};
use uuid::Uuid;

use error::CommandError;
use storage::{expand_home, read_library, read_settings, write_library, write_settings};
use types::{
    AddGameInput, Game, GameDiscovery, GameLog, GamePreflight, GameSetupPlan, LaunchAttempt,
    LaunchState, LibraryFile, PrefixStatus, Runtime, RuntimeKind, Settings, SystemDiagnostics,
};

struct AppState {
    launch: Arc<LaunchRegistry>,
}

fn now_iso() -> String {
    // Sufficient for local library records; not used for crypto.
    format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    )
}

fn prefix_path_for(dir: &str, id: &str) -> String {
    let base = dir.trim_end_matches('/').trim_end_matches('\\');
    format!("{base}/{id}")
}

fn find_game_mut<'a>(library: &'a mut LibraryFile, id: &str) -> Result<&'a mut Game, CommandError> {
    library
        .games
        .iter_mut()
        .find(|game| game.id == id)
        .ok_or_else(|| CommandError::new("not_found", "Jogo não encontrado."))
}

fn find_game(library: &LibraryFile, id: &str) -> Result<Game, CommandError> {
    library
        .games
        .iter()
        .find(|game| game.id == id)
        .cloned()
        .ok_or_else(|| CommandError::new("not_found", "Jogo não encontrado."))
}

#[tauri::command]
fn detect_runtimes() -> Vec<Runtime> {
    runtimes::detect_runtimes()
}

#[tauri::command]
fn list_games() -> Result<Vec<Game>, CommandError> {
    Ok(read_library()?.games)
}

#[tauri::command]
fn get_game(id: String) -> Result<Option<Game>, CommandError> {
    Ok(read_library()?.games.into_iter().find(|game| game.id == id))
}

#[tauri::command]
fn add_game(input: AddGameInput, state: State<AppState>) -> Result<Game, CommandError> {
    let discovery = discover::resolve_game_path(&input.folder_path)?;
    let name = {
        let trimmed = input.name.trim();
        if trimmed.is_empty() {
            discovery.suggested_name.clone()
        } else {
            trimmed.to_string()
        }
    };
    let settings = read_settings()?;
    let id = Uuid::new_v4().to_string();
    let game = Game {
        id: id.clone(),
        name,
        exe_path: discovery.exe_path.clone(),
        install_dir: discovery.folder_path.clone(),
        args: input.args.trim().to_string(),
        runtime_kind: discovery.recommended_runtime,
        dx_backend: discovery.recommended_backend,
        prefix_path: prefix_path_for(&settings.prefixes_dir, &id),
        prefix_status: PrefixStatus::Missing,
        prefix_error: None,
        created_at: now_iso(),
    };
    let mut library = read_library()?;
    library.games.push(game.clone());
    write_library(&library)?;
    launch::append_log(
        state.launch.as_ref(),
        &id,
        &format!("Pasta do jogo: {}", discovery.folder_path),
    );
    launch::append_log(
        state.launch.as_ref(),
        &id,
        &format!("Executável encontrado: {}", discovery.exe_path),
    );
    Ok(game)
}

#[tauri::command]
fn resolve_game_folder(path: String) -> Result<GameDiscovery, CommandError> {
    discover::resolve_game_path(&path)
}

#[tauri::command]
fn update_game(game: Game) -> Result<Game, CommandError> {
    let mut library = read_library()?;
    let stored = find_game_mut(&mut library, &game.id)?;
    *stored = game.clone();
    write_library(&library)?;
    Ok(game)
}

#[tauri::command]
fn remove_game(id: String, state: State<AppState>) -> Result<(), CommandError> {
    let _ = launch::stop_game(state.launch.as_ref(), &id);
    let mut library = read_library()?;
    library.games.retain(|game| game.id != id);
    write_library(&library)
}

fn picked_path(path: Option<FilePath>) -> Option<String> {
    path.map(|path| path.simplified().to_string())
}

#[tauri::command]
async fn pick_exe(app: tauri::AppHandle, window: tauri::Window) -> Option<String> {
    picked_path(
        app.dialog()
            .file()
            .set_parent(&window)
            .set_title("Escolher executável Windows")
            .add_filter("Executável Windows", &["exe"])
            .blocking_pick_file(),
    )
}

#[tauri::command]
async fn pick_wine(app: tauri::AppHandle, window: tauri::Window) -> Option<String> {
    picked_path(
        app.dialog()
            .file()
            .set_parent(&window)
            .set_title("Escolher wine64")
            .blocking_pick_file(),
    )
}

#[tauri::command]
async fn pick_directory(app: tauri::AppHandle, window: tauri::Window) -> Option<String> {
    picked_path(
        app.dialog()
            .file()
            .set_parent(&window)
            .set_title("Escolher pasta de prefixos")
            .blocking_pick_folder(),
    )
}

#[tauri::command]
async fn pick_game_folder(app: tauri::AppHandle, window: tauri::Window) -> Option<String> {
    picked_path(
        app.dialog()
            .file()
            .set_parent(&window)
            .set_title("Escolher pasta do jogo")
            .blocking_pick_folder(),
    )
}

fn apply_prefix(state: &AppState, game_id: &str, wipe: bool) -> Result<Game, CommandError> {
    if wipe {
        let _ = launch::stop_game(state.launch.as_ref(), game_id);
    }
    let mut library = read_library()?;
    let game = find_game(&library, game_id)?;
    match launch::create_prefix(state.launch.as_ref(), &game, wipe) {
        Ok(()) => {
            let stored = find_game_mut(&mut library, game_id)?;
            stored.prefix_status = PrefixStatus::Ready;
            stored.prefix_error = None;
            let updated = stored.clone();
            write_library(&library)?;
            Ok(updated)
        }
        Err(err) => {
            let stored = find_game_mut(&mut library, game_id)?;
            stored.prefix_status = PrefixStatus::Error;
            stored.prefix_error = Some(err.message.clone());
            write_library(&library)?;
            Err(err)
        }
    }
}

#[tauri::command]
fn create_prefix(
    game_id: String,
    wipe: bool,
    state: State<AppState>,
) -> Result<Game, CommandError> {
    apply_prefix(&state, &game_id, wipe)
}

#[tauri::command]
fn get_game_setup(id: String) -> Result<GameSetupPlan, CommandError> {
    let game = find_game(&read_library()?, &id)?;
    Ok(setup::plan_for_game(&game))
}

#[tauri::command]
fn get_system_diagnostics() -> SystemDiagnostics {
    diagnostics::collect()
}

#[tauri::command]
fn get_game_preflight(id: String) -> Result<GamePreflight, CommandError> {
    let game = find_game(&read_library()?, &id)?;
    Ok(preflight::inspect(&game))
}

#[tauri::command]
fn get_launch_attempts(
    game_id: String,
    limit: Option<usize>,
) -> Result<Vec<LaunchAttempt>, CommandError> {
    let mut attempts = attempts::list(&game_id)?;
    if let Some(limit) = limit {
        attempts.truncate(limit);
    }
    Ok(attempts)
}

#[tauri::command]
fn list_launch_attempts(
    game_id: String,
    limit: Option<usize>,
) -> Result<Vec<LaunchAttempt>, CommandError> {
    get_launch_attempts(game_id, limit)
}

#[tauri::command]
fn get_launch_attempt(game_id: String, attempt_id: String) -> Result<LaunchAttempt, CommandError> {
    attempts::read(&game_id, &attempt_id)
}

#[tauri::command]
fn get_launch_attempt_log(game_id: String, attempt_id: String) -> Result<GameLog, CommandError> {
    attempts::log(&game_id, &attempt_id)
}

#[tauri::command(async)]
fn prepare_game(game_id: String, state: State<AppState>) -> Result<Game, CommandError> {
    setup::prepare_game(state.launch.as_ref(), &game_id)
}

#[tauri::command(async)]
fn install_runtime(kind: RuntimeKind) -> Result<Vec<Runtime>, CommandError> {
    setup::install_runtime(&kind)?;
    Ok(runtimes::detect_runtimes())
}

#[tauri::command(async)]
fn launch_game(game_id: String, state: State<AppState>) -> Result<LaunchAttempt, CommandError> {
    let library = read_library()?;
    let game = find_game(&library, &game_id)?;
    let preflight = preflight::inspect(&game);
    if preflight.status == types::PreflightStatus::Blocked {
        let reason = preflight
            .checks
            .iter()
            .find(|check| check.status == types::CheckStatus::Fail)
            .map(|check| check.detail.clone())
            .unwrap_or_else(|| "O preflight bloqueou este lançamento.".to_string());
        let _ = attempts::record_failure(
            &game,
            types::LaunchAttemptStatus::Blocked,
            "launch_blocked",
            &reason,
            Some(preflight.fingerprint),
        );
        return Err(CommandError::new("launch_blocked", reason));
    }
    let setup_plan = setup::plan_for_game(&game);
    let stack_changed = game.runtime_kind != setup_plan.recommended_runtime
        || game.dx_backend != setup_plan.recommended_backend;
    if !setup::game_is_ready(&game) || stack_changed {
        if let Err(error) = setup::prepare_game(state.launch.as_ref(), &game_id) {
            let _ = attempts::record_failure(
                &game,
                types::LaunchAttemptStatus::Failed,
                &error.code,
                &error.message,
                Some(preflight.fingerprint),
            );
            return Err(error);
        }
    }
    let library = read_library()?;
    let game = find_game(&library, &game_id)?;
    launch::launch_game(Arc::clone(&state.launch), &game)
}

#[tauri::command]
fn stop_game(game_id: String, state: State<AppState>) -> Result<(), CommandError> {
    launch::stop_game(state.launch.as_ref(), &game_id)
}

#[tauri::command]
fn install_vcrun(game_id: String, state: State<AppState>) -> Result<(), CommandError> {
    let library = read_library()?;
    let game = find_game(&library, &game_id)?;
    if game.prefix_status != PrefixStatus::Ready {
        apply_prefix(&state, &game_id, false)?;
    }
    let library = read_library()?;
    let game = find_game(&library, &game_id)?;
    launch::install_vcrun(state.launch.as_ref(), &game)
}

#[tauri::command]
fn get_logs(game_id: String, state: State<AppState>) -> GameLog {
    launch::logs_for(state.launch.as_ref(), &game_id)
}

#[tauri::command]
fn get_launch_state(game_id: String, state: State<AppState>) -> LaunchState {
    launch::current_state(state.launch.as_ref(), &game_id)
}

#[tauri::command]
fn get_settings() -> Result<Settings, CommandError> {
    read_settings()
}

#[tauri::command]
fn save_settings(settings: Settings) -> Result<Settings, CommandError> {
    let mut next = settings;
    next.custom_wine_path = next.custom_wine_path.trim().to_string();
    if next.prefixes_dir.trim().is_empty() {
        next.prefixes_dir = Settings::default().prefixes_dir;
    }
    let _ = expand_home(&next.prefixes_dir);
    write_settings(&next)?;
    Ok(next)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            launch: Arc::new(LaunchRegistry::new()),
        })
        .invoke_handler(tauri::generate_handler![
            detect_runtimes,
            list_games,
            get_game,
            add_game,
            update_game,
            remove_game,
            pick_exe,
            pick_wine,
            pick_directory,
            pick_game_folder,
            resolve_game_folder,
            create_prefix,
            get_game_setup,
            get_system_diagnostics,
            get_game_preflight,
            get_launch_attempts,
            list_launch_attempts,
            get_launch_attempt,
            get_launch_attempt_log,
            prepare_game,
            install_runtime,
            launch_game,
            stop_game,
            install_vcrun,
            get_logs,
            get_launch_state,
            get_settings,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
