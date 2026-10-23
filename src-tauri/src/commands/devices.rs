use serde::Serialize;

use crate::error::Result;
use crate::ffmpeg;
use crate::ffmpeg::devices::CaptureDevice;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegInfo {
    available: bool,
    version: Option<String>,
}

/// FFmpeg est-il livré et répond-il ?
#[tauri::command]
pub async fn ffmpeg_info() -> FfmpegInfo {
    let version = tauri::async_runtime::spawn_blocking(|| ffmpeg::version().ok())
        .await
        .ok()
        .flatten();
    FfmpegInfo {
        available: version.is_some(),
        version,
    }
}

/// Périphériques de capture vidéo et audio branchés.
#[tauri::command]
pub async fn list_capture_devices() -> Result<Vec<CaptureDevice>> {
    tauri::async_runtime::spawn_blocking(ffmpeg::devices::list)
        .await
        .map_err(|e| crate::error::Error::Tool {
            program: "ffmpeg",
            message: e.to_string(),
        })?
}
