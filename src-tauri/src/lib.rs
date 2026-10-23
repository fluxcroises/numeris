//! Cœur de Numéris : pilotage de FFmpeg, catalogue, bibliothèque et tâches.

mod commands;
mod db;
mod error;
mod ffmpeg;
mod library;
mod settings;

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use tauri::Manager;

use library::Library;
use settings::Settings;

/// Variable d'environnement de l'essai de lancement de la CI : l'application
/// s'arrête d'elle-même dès que l'interface est affichée.
const SMOKE_ENV: &str = "NUMERIS_SMOKE";

/// État partagé par les commandes.
pub struct AppState {
    settings_path: PathBuf,
    settings: Mutex<Settings>,
    /// Bibliothèque ouverte ; absente tant qu'aucun dossier n'est choisi ou
    /// quand son disque n'est pas branché.
    library: Mutex<Option<Library>>,
}

impl AppState {
    fn settings(&self) -> MutexGuard<'_, Settings> {
        self.settings.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn library(&self) -> MutexGuard<'_, Option<Library>> {
        self.library.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Appelée par l'interface après sa première image.
#[tauri::command]
fn app_ready(app: tauri::AppHandle) {
    if std::env::var_os(SMOKE_ENV).is_some() {
        app.exit(0);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // FFmpeg est livré dans les ressources de l'application (tauri.conf.json).
            if let Ok(dir) = app.path().resource_dir() {
                ffmpeg::set_dir(dir.join("ffmpeg"));
            }
            let settings_path = app.path().app_config_dir()?.join("settings.json");
            let settings = Settings::load(&settings_path);
            // Un disque débranché laisse la bibliothèque fermée : l'interface le signale.
            let library = settings
                .library_dir
                .as_deref()
                .filter(|dir| dir.is_dir())
                .and_then(|dir| Library::open(dir).ok());
            app.manage(AppState {
                settings_path,
                settings: Mutex::new(settings),
                library: Mutex::new(library),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_ready,
            commands::devices::ffmpeg_info,
            commands::devices::list_capture_devices,
            commands::library::get_settings,
            commands::library::set_library_dir,
            commands::library::set_language,
            commands::library::list_cassettes,
            commands::library::create_cassette,
        ])
        .run(tauri::generate_context!())
        .expect("échec du lancement de Numéris");
}
