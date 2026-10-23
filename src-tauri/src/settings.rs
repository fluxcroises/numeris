//! Réglages de l'application, propres à l'ordinateur : dossier de la
//! bibliothèque et langue. Fichier JSON dans le dossier de configuration.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub library_dir: Option<PathBuf>,
    /// « fr » ou « en » ; absent, la langue du système.
    pub language: Option<String>,
}

pub const LANGUAGES: &[&str] = &["fr", "en"];

impl Settings {
    /// Lit les réglages ; un fichier absent ou illisible donne les réglages par défaut.
    pub fn load(path: &Path) -> Settings {
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// Écrit les réglages d'un coup (fichier temporaire renommé).
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        fs::rename(&tmp, path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config").join("settings.json");
        assert_eq!(Settings::load(&path), Settings::default());
        let settings = Settings {
            library_dir: Some(dir.path().join("Numeris")),
            language: Some("en".into()),
        };
        settings.save(&path).unwrap();
        assert_eq!(Settings::load(&path), settings);
    }

    #[test]
    fn unreadable_file_gives_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, b"{ pas du json").unwrap();
        assert_eq!(Settings::load(&path), Settings::default());
    }
}
