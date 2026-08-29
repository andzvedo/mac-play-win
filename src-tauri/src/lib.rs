mod env_backend;
mod error;
mod launch;
mod runtimes;
mod storage;
mod types;

use std::sync::Arc;

use launch::LaunchRegistry;
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

use error::CommandError;
use storage::{expand_home, read_library, read_settings, write_library, write_settings};
use types::{
    AddGameInput, Game, GameLog, LaunchState, LibraryFile, PrefixStatus, Runtime, Settings,
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
fn add_game(input: AddGameInput) -> Result<Game, CommandError> {
    let name = input.name.trim().to_string();
    let exe_path = input.exe_path.trim().to_string();
    if name.is_empty() || exe_path.is_empty() {
        return Err(CommandError::new(
            "invalid_exe",
            "Informe o nome e o caminho do .exe.",
        ));
    }
    let settings = read_settings()?;
    let id = Uuid::new_v4().to_string();
    let game = Game {
        id: id.clone(),
        name,
        exe_path,
        args: input.args.trim().to_string(),
        runtime_kind: input.runtime_kind,
        dx_backend: input.dx_backend,
        prefix_path: prefix_path_for(&settings.prefixes_dir, &id),
        prefix_status: PrefixStatus::Missing,
        prefix_error: None,
        created_at: now_iso(),
    };
    let mut library = read_library()?;
    library.games.push(game.clone());
    write_library(&library)?;
    Ok(game)
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

#[tauri::command]
fn pick_exe(app: tauri::AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .add_filter("Executável Windows", &["exe"])
        .blocking_pick_file()
        .map(|path| path.to_string())
}

fn apply_prefix(state: &AppState, game_id: &str) -> Result<Game, CommandError> {
    let mut library = read_library()?;
    let game = find_game(&library, game_id)?;
    match launch::create_prefix(state.launch.as_ref(), &game) {
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
fn create_prefix(game_id: String, state: State<AppState>) -> Result<Game, CommandError> {
    apply_prefix(&state, &game_id)
}

#[tauri::command]
fn launch_game(game_id: String, state: State<AppState>) -> Result<(), CommandError> {
    let library = read_library()?;
    let game = find_game(&library, &game_id)?;
    if game.prefix_status != PrefixStatus::Ready {
        apply_prefix(&state, &game_id)?;
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
            create_prefix,
            launch_game,
            stop_game,
            get_logs,
            get_launch_state,
            get_settings,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
