use serde::{Deserialize, Serialize};
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::AppState;
use crate::capture::{CaptureConfig, CaptureInput, CaptureOutcome, Session, finalize};
use crate::error::{Error, Result};
use crate::ffmpeg::devices::{self, VideoInput};
use crate::ffmpeg::probe::probe;
use crate::library::{CassetteSummary, MasterRecord};

const MASTER_FILE: &str = "master.mkv";
/// Marge gardée libre sur le disque : la capture s'arrête proprement en dessous.
const MIN_FREE_BYTES: u64 = 2_000_000_000;
/// Estimation d'une heure de master FFV1 en PAL 4:2:2, à affiner sur de vraies
/// cassettes ; pendant la capture, l'interface calcule le débit réel.
const BYTES_PER_HOUR: u64 = 25_000_000_000;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartCapture {
    cassette_id: i64,
    input: CaptureInput,
    end_of_tape_after_s: Option<u32>,
    max_duration_s: Option<u32>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureFinished {
    outcome: CaptureOutcome,
    cassette: Option<CassetteSummary>,
    error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureSpace {
    free_bytes: u64,
    bytes_per_hour: u64,
    min_free_bytes: u64,
}

/// Entrées vidéo (composite, S-Vidéo) d'un boîtier.
#[tauri::command]
pub async fn list_video_inputs(device: String) -> Result<Vec<VideoInput>> {
    tauri::async_runtime::spawn_blocking(move || devices::video_inputs(&device))
        .await
        .map_err(|e| Error::Invalid(e.to_string()))?
}

/// Place libre pour le master d'une cassette, et ce qu'occupe une heure.
#[tauri::command]
pub fn capture_space(state: State<AppState>, cassette_id: i64) -> Result<CaptureSpace> {
    let (dir, _) = state
        .library()
        .as_ref()
        .ok_or(Error::NoLibrary)?
        .cassette_dir(cassette_id)?;
    Ok(CaptureSpace {
        free_bytes: fs4::available_space(&dir)?,
        bytes_per_hour: BYTES_PER_HOUR,
        min_free_bytes: MIN_FREE_BYTES,
    })
}

/// Lance la capture d'une cassette. L'aperçu arrive par `preview` (images
/// JPEG brutes), les mesures par l'événement `capture-telemetry`, l'avancement
/// des empreintes par `capture-finalizing`, la fin par `capture-finished`.
#[tauri::command]
pub fn start_capture(
    app: AppHandle,
    state: State<AppState>,
    request: StartCapture,
    preview: Channel<InvokeResponseBody>,
) -> Result<()> {
    let mut current = state.capture();
    if current.is_some() {
        return Err(Error::Invalid("une capture est déjà en cours".into()));
    }
    let (dir, standard) = state
        .library()
        .as_ref()
        .ok_or(Error::NoLibrary)?
        .cassette_dir(request.cassette_id)?;
    let master = dir.join(MASTER_FILE);
    // Règle absolue : un master n'est jamais remplacé.
    if master.exists() {
        return Err(Error::Invalid(
            "cette cassette a déjà un master, qui ne sera pas remplacé".into(),
        ));
    }

    let config = CaptureConfig {
        input: request.input,
        standard,
        master: master.clone(),
        preview_width: 480,
        end_of_tape_after_s: request.end_of_tape_after_s,
        max_duration_s: request.max_duration_s,
        min_free_bytes: MIN_FREE_BYTES,
    };
    let telemetry_app = app.clone();
    let session = Session::start(
        config,
        move |jpeg| {
            let _ = preview.send(InvokeResponseBody::Raw(jpeg));
        },
        move |t| {
            let _ = telemetry_app.emit("capture-telemetry", t);
        },
    )?;
    *current = Some(session.stopper());
    drop(current);

    let cassette_id = request.cassette_id;
    std::thread::spawn(move || {
        let outcome = session.wait();
        let state = app.state::<AppState>();
        *state.capture() = None;
        let result = finish(&app, &state, cassette_id, &master, &outcome);
        let (cassette, error) = match result {
            Ok(c) => (Some(c), outcome.error.clone()),
            Err(e) => (None, Some(outcome.error.clone().unwrap_or(e.to_string()))),
        };
        let _ = app.emit(
            "capture-finished",
            CaptureFinished {
                outcome,
                cassette,
                error,
            },
        );
    });
    Ok(())
}

/// Empreintes du master et enregistrement dans la bibliothèque.
fn finish(
    app: &AppHandle,
    state: &AppState,
    cassette_id: i64,
    master: &std::path::Path,
    outcome: &CaptureOutcome,
) -> Result<CassetteSummary> {
    let info = probe(master)?;
    let duration_ms = info.duration_ms.unwrap_or_default();
    if duration_ms == 0 {
        return Err(Error::Invalid("le master est vide".into()));
    }
    let prints = finalize::fingerprints(master, duration_ms, |p| {
        let _ = app.emit("capture-finalizing", p);
    })?;
    let framemd5 = prints
        .framemd5
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_default();
    state
        .library()
        .as_mut()
        .ok_or(Error::NoLibrary)?
        .record_master(
            cassette_id,
            MasterRecord {
                file: MASTER_FILE.into(),
                blake3: prints.blake3,
                framemd5,
                duration_ms,
                field_order: outcome.field_order.clone(),
            },
        )
}

/// Demande l'arrêt propre de la capture en cours.
#[tauri::command]
pub fn stop_capture(state: State<AppState>) {
    if let Some(stopper) = state.capture().as_ref() {
        stopper.stop();
    }
}
