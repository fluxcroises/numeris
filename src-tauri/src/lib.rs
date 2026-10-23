//! Cœur de Numéris : pilotage de FFmpeg, catalogue, bibliothèque et tâches.

mod commands;
mod error;
mod ffmpeg;

use tauri::Manager;

/// Variable d'environnement de l'essai de lancement de la CI : l'application
/// s'arrête d'elle-même dès que l'interface est affichée.
const SMOKE_ENV: &str = "NUMERIS_SMOKE";

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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_ready,
            commands::devices::ffmpeg_info,
            commands::devices::list_capture_devices,
        ])
        .run(tauri::generate_context!())
        .expect("échec du lancement de Numéris");
}
