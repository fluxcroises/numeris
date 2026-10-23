//! Erreurs du cœur, transmises à l'interface sous forme de texte.

use serde::Serialize;

use crate::db::DbError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("FFmpeg est introuvable")]
    FfmpegMissing,
    #[error("échec de {program} : {message}")]
    Tool {
        program: &'static str,
        message: String,
    },
    #[error("aucune bibliothèque n'est choisie")]
    NoLibrary,
    #[error("{0}")]
    Invalid(String),
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Db(DbError::Sqlite(e))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

// L'interface reçoit le message, pas la structure.
impl Serialize for Error {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
