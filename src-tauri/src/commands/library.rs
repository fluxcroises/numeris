use std::path::PathBuf;

use tauri::State;

use crate::AppState;
use crate::error::{Error, Result};
use crate::library::{CassetteSummary, Library, NewCassette};
use crate::settings::{LANGUAGES, Settings};

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Settings {
    state.settings().clone()
}

/// Choisit (ou crée) le dossier de la bibliothèque et le retient.
#[tauri::command]
pub fn set_library_dir(state: State<AppState>, path: PathBuf) -> Result<Settings> {
    let library = Library::open(&path)?;
    *state.library() = Some(library);
    let mut settings = state.settings();
    settings.library_dir = Some(path);
    settings.save(&state.settings_path)?;
    Ok(settings.clone())
}

/// Langue de l'interface ; `None` suit la langue du système.
#[tauri::command]
pub fn set_language(state: State<AppState>, language: Option<String>) -> Result<Settings> {
    if let Some(lang) = &language
        && !LANGUAGES.contains(&lang.as_str())
    {
        return Err(Error::Invalid(format!("langue inconnue : {lang}")));
    }
    let mut settings = state.settings();
    settings.language = language;
    settings.save(&state.settings_path)?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn list_cassettes(state: State<AppState>) -> Result<Vec<CassetteSummary>> {
    state
        .library()
        .as_ref()
        .ok_or(Error::NoLibrary)?
        .cassettes()
}

#[tauri::command]
pub fn create_cassette(state: State<AppState>, cassette: NewCassette) -> Result<CassetteSummary> {
    state
        .library()
        .as_mut()
        .ok_or(Error::NoLibrary)?
        .create_cassette(cassette)
}
