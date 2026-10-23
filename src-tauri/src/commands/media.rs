use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::{Error, Result};
use crate::ffmpeg::probe::{MediaInfo, probe};
use crate::ffmpeg::proxy;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenedVideo {
    info: MediaInfo,
    /// Fichier à donner au lecteur ; absent tant que la copie de lecture se prépare.
    play_path: Option<PathBuf>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProxyEvent {
    source: PathBuf,
    progress: f64,
    play_path: Option<PathBuf>,
    error: Option<String>,
}

/// Le lecteur n'accède qu'aux fichiers ouverts par l'utilisateur et à leurs copies.
fn allow(app: &AppHandle, path: &Path) -> Result<()> {
    app.asset_protocol_scope()
        .allow_file(path)
        .map_err(|e| Error::Invalid(e.to_string()))
}

/// Copie de lecture d'un fichier hors catalogue, dans le cache : son nom dépend
/// du chemin, de la taille et de la date du fichier, pour suivre ses changements.
fn cached_proxy(app: &AppHandle, source: &Path) -> Result<PathBuf> {
    let meta = std::fs::metadata(source)?;
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    meta.len().hash(&mut hasher);
    meta.modified().ok().hash(&mut hasher);
    let dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| Error::Invalid(e.to_string()))?
        .join("proxies");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join(format!("{:016x}.mp4", hasher.finish())))
}

/// Ouvre une vidéo pour le lecteur ; prépare au besoin sa copie de lecture en
/// arrière-plan (événements `proxy-progress`).
#[tauri::command]
pub async fn open_video(app: AppHandle, path: PathBuf) -> Result<OpenedVideo> {
    let source = path.clone();
    let info = tauri::async_runtime::spawn_blocking(move || probe(&source))
        .await
        .map_err(|e| Error::Invalid(e.to_string()))??;

    if info.playable {
        allow(&app, &path)?;
        return Ok(OpenedVideo {
            info,
            play_path: Some(path),
        });
    }
    let out = cached_proxy(&app, &path)?;
    if out.is_file() {
        allow(&app, &out)?;
        return Ok(OpenedVideo {
            info,
            play_path: Some(out),
        });
    }

    let job_info = info.clone();
    std::thread::spawn(move || {
        let emit = |progress: f64, play_path: Option<PathBuf>, error: Option<String>| {
            let _ = app.emit(
                "proxy-progress",
                ProxyEvent {
                    source: path.clone(),
                    progress,
                    play_path,
                    error,
                },
            );
        };
        let mut last = -1.0;
        let result = proxy::make(&path, &job_info, &out, |p| {
            // Un événement par pour cent suffit à l'interface.
            if p - last >= 0.01 {
                last = p;
                emit(p, None, None);
            }
        });
        match result.and_then(|()| allow(&app, &out)) {
            Ok(()) => emit(1.0, Some(out), None),
            Err(e) => emit(0.0, None, Some(e.to_string())),
        }
    });
    Ok(OpenedVideo {
        info,
        play_path: None,
    })
}
