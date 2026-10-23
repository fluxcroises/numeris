//! Capture analogique : une seule commande FFmpeg écrit le master FFV1 + PCM
//! en MKV, produit l'aperçu MJPEG (sur sa sortie standard) et imprime les
//! mesures (sur sa sortie d'erreur). Sous Windows, un seul programme peut
//! ouvrir le boîtier : l'aperçu ne peut venir que de FFmpeg lui-même.
//!
//! L'arrêt envoie « q » à FFmpeg, qui termine proprement le MKV ; un MKV
//! interrompu brutalement reste lisible jusqu'à la coupure.

pub mod finalize;
mod mjpeg;
pub mod telemetry;

use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::ffmpeg;
use crate::library::cassette::TvStandard;
use telemetry::{Parser, SignalState, Telemetry};

/// Source de la capture.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum CaptureInput {
    /// Windows : noms uniques DirectShow de la vidéo et du son, et broche
    /// d'entrée du boîtier (composite, S-Vidéo) quand il en propose plusieurs.
    #[serde(rename_all = "camelCase")]
    DirectShow {
        video: String,
        audio: Option<String>,
        video_pin: Option<u32>,
    },
    /// macOS : indices AVFoundation.
    #[serde(rename_all = "camelCase")]
    AvFoundation {
        video: String,
        audio: Option<String>,
    },
    /// Mire et son générés par FFmpeg, au rythme réel : essais sans matériel.
    /// `video` est un graphe lavfi (une mire suivie de noir, par exemple).
    #[cfg(test)]
    Generated { video: String, audio: bool },
}

#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub input: CaptureInput,
    pub standard: TvStandard,
    /// Fichier master à écrire (`…/master.mkv`).
    pub master: PathBuf,
    /// Largeur de l'aperçu en pixels.
    pub preview_width: u32,
    /// Arrêt après ce temps de noir ou de neige, une fois l'image apparue.
    pub end_of_tape_after_s: Option<u32>,
    /// Durée maximale (minuterie).
    pub max_duration_s: Option<u32>,
    /// Arrêt propre quand l'espace libre passe sous ce seuil.
    pub min_free_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StopReason {
    User,
    EndOfTape,
    Timer,
    DiskFull,
    /// La source s'est arrêtée d'elle-même (boîtier débranché, fin d'un essai).
    InputEnded,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureOutcome {
    pub reason: StopReason,
    pub last: Option<Telemetry>,
    /// Ordre des trames estimé (« tff », « bff », « progressive »).
    pub field_order: Option<String>,
    /// Message de FFmpeg quand la capture a échoué.
    pub error: Option<String>,
}

/// Capture en cours.
pub struct Session {
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    child: Arc<Mutex<Child>>,
    reason: Arc<Mutex<Option<StopReason>>>,
    monitor: JoinHandle<CaptureOutcome>,
}

/// Délai laissé à FFmpeg pour finir le MKV après « q » avant de l'arrêter de force.
const STOP_GRACE: Duration = Duration::from_secs(15);

fn request_stop(
    stdin: &Mutex<Option<ChildStdin>>,
    child: &Arc<Mutex<Child>>,
    reason: &Mutex<Option<StopReason>>,
    why: StopReason,
) {
    {
        let mut current = reason.lock().unwrap_or_else(|e| e.into_inner());
        if current.is_some() {
            return;
        }
        *current = Some(why);
    }
    if let Some(mut pipe) = stdin.lock().unwrap_or_else(|e| e.into_inner()).take() {
        let _ = pipe.write_all(b"q");
        let _ = pipe.flush();
    }
    let child = Arc::clone(child);
    std::thread::spawn(move || {
        std::thread::sleep(STOP_GRACE);
        // Sans effet si FFmpeg s'est déjà arrêté.
        let _ = child.lock().unwrap_or_else(|e| e.into_inner()).kill();
    });
}

impl Session {
    /// Lance la capture. `on_preview` reçoit chaque image JPEG de l'aperçu,
    /// `on_telemetry` un instantané des mesures deux fois par seconde.
    pub fn start(
        config: CaptureConfig,
        mut on_preview: impl FnMut(Vec<u8>) + Send + 'static,
        mut on_telemetry: impl FnMut(&Telemetry) + Send + 'static,
    ) -> Result<Session> {
        let mut child = ffmpeg::ffmpeg()?
            .args(args(&config))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdin = Arc::new(Mutex::new(child.stdin.take()));
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let child = Arc::new(Mutex::new(child));
        let reason: Arc<Mutex<Option<StopReason>>> = Arc::new(Mutex::new(None));

        let preview = stdout.map(|mut out| {
            std::thread::spawn(move || {
                let mut splitter = mjpeg::FrameSplitter::default();
                let mut buf = vec![0; 64 * 1024];
                while let Ok(n) = out.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    for frame in splitter.push(&buf[..n]) {
                        on_preview(frame);
                    }
                }
            })
        });

        let monitor = {
            let (stdin, child, reason) = (stdin.clone(), child.clone(), reason.clone());
            std::thread::spawn(move || {
                let mut parser = Parser::default();
                let mut last: Option<Telemetry> = None;
                let mut seen_picture = false;
                let mut bad_since: Option<u64> = None;
                let mut free_bytes = None;
                let dir = config
                    .master
                    .parent()
                    .map(PathBuf::from)
                    .unwrap_or_default();
                let mut snapshots = 0u64;
                if let Some(err) = stderr {
                    for line in BufReader::new(err).lines().map_while(std::io::Result::ok) {
                        let Some(mut t) = parser.line(&line) else {
                            continue;
                        };
                        // Espace libre : vérifié toutes les 5 secondes environ.
                        if snapshots.is_multiple_of(10) {
                            free_bytes = fs4::available_space(&dir).ok();
                        }
                        snapshots += 1;
                        t.free_bytes = free_bytes;

                        let why = auto_stop(&config, &t, &mut seen_picture, &mut bad_since);
                        if let Some(why) = why {
                            request_stop(&stdin, &child, &reason, why);
                        }
                        on_telemetry(&t);
                        last = Some(t);
                    }
                }
                let status = child.lock().unwrap_or_else(|e| e.into_inner()).wait();
                if let Some(preview) = preview {
                    let _ = preview.join();
                }
                let reason = reason
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .unwrap_or(StopReason::InputEnded);
                // Un arrêt demandé termine FFmpeg normalement ; sinon, un code
                // d'échec avec des erreurs explique ce qui s'est passé.
                let failed = !status.as_ref().is_ok_and(|s| s.success());
                let error = (failed && !parser.errors.is_empty()).then(|| parser.errors.join("\n"));
                CaptureOutcome {
                    reason,
                    field_order: last
                        .as_ref()
                        .and_then(|t| t.fields.guess())
                        .map(str::to_owned),
                    last,
                    error,
                }
            })
        };

        Ok(Session {
            stdin,
            child,
            reason,
            monitor,
        })
    }

    /// Demande l'arrêt propre de la capture.
    #[cfg(test)]
    pub fn stop(&self) {
        self.stopper().stop();
    }

    /// Poignée qui permet d'arrêter la capture pendant qu'un autre fil l'attend.
    pub fn stopper(&self) -> Stopper {
        Stopper {
            stdin: self.stdin.clone(),
            child: self.child.clone(),
            reason: self.reason.clone(),
        }
    }

    /// Attend la fin de FFmpeg.
    pub fn wait(self) -> CaptureOutcome {
        self.monitor.join().unwrap_or_else(|_| CaptureOutcome {
            reason: StopReason::InputEnded,
            last: None,
            field_order: None,
            error: Some("la surveillance de la capture s'est arrêtée".into()),
        })
    }
}

#[derive(Clone)]
pub struct Stopper {
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    child: Arc<Mutex<Child>>,
    reason: Arc<Mutex<Option<StopReason>>>,
}

impl Stopper {
    pub fn stop(&self) {
        request_stop(&self.stdin, &self.child, &self.reason, StopReason::User);
    }
}

/// Faut-il arrêter la capture d'après ce dernier instantané ?
fn auto_stop(
    config: &CaptureConfig,
    t: &Telemetry,
    seen_picture: &mut bool,
    bad_since: &mut Option<u64>,
) -> Option<StopReason> {
    if t.free_bytes
        .is_some_and(|free| free < config.min_free_bytes)
    {
        return Some(StopReason::DiskFull);
    }
    if config
        .max_duration_s
        .is_some_and(|max| t.elapsed_ms >= u64::from(max) * 1000)
    {
        return Some(StopReason::Timer);
    }
    match t.signal {
        SignalState::Ok => {
            *seen_picture = true;
            *bad_since = None;
        }
        SignalState::Black | SignalState::Snow => {
            let since = *bad_since.get_or_insert(t.elapsed_ms);
            // La fin de bande ne compte qu'après le début de l'enregistrement :
            // le noir du début (amorce, rembobinage) ne déclenche rien.
            if *seen_picture
                && let Some(after) = config.end_of_tape_after_s
                && t.elapsed_ms - since >= u64::from(after) * 1000
            {
                return Some(StopReason::EndOfTape);
            }
        }
        SignalState::Unknown => {}
    }
    None
}

fn standard_params(standard: TvStandard) -> (&'static str, &'static str) {
    match standard {
        TvStandard::Pal | TvStandard::Secam => ("720x576", "25"),
        TvStandard::Ntsc => ("720x480", "30000/1001"),
    }
}

/// Arguments de FFmpeg pour une capture.
fn args(config: &CaptureConfig) -> Vec<OsString> {
    let (size, rate) = standard_params(config.standard);
    let mut a: Vec<OsString> = Vec::new();
    let mut push = |items: &[&str]| a.extend(items.iter().map(OsString::from));
    push(&[
        "-hide_banner",
        "-nostats",
        "-loglevel",
        "level+info",
        "-progress",
        "pipe:2",
        "-stats_period",
        "0.5",
    ]);

    // Entrées : où trouver la vidéo et le son dans la suite.
    let (video_ref, audio_ref): (&str, Option<&str>) = match &config.input {
        CaptureInput::DirectShow {
            video,
            audio,
            video_pin,
        } => {
            push(&[
                "-f",
                "dshow",
                "-thread_queue_size",
                "1024",
                "-rtbufsize",
                "1024M",
                "-video_size",
                size,
                "-framerate",
                rate,
                "-pixel_format",
                "yuyv422",
            ]);
            if let Some(pin) = video_pin {
                push(&["-crossbar_video_input_pin_number", &pin.to_string()]);
            }
            let mut spec = format!("video={video}");
            if let Some(audio) = audio {
                push(&[
                    "-sample_rate",
                    "48000",
                    "-sample_size",
                    "16",
                    "-channels",
                    "2",
                    "-audio_buffer_size",
                    "40",
                ]);
                spec.push_str(&format!(":audio={audio}"));
            }
            push(&["-i", &spec]);
            ("[0:v]", audio.as_ref().map(|_| "[0:a]"))
        }
        CaptureInput::AvFoundation { video, audio } => {
            push(&[
                "-f",
                "avfoundation",
                "-thread_queue_size",
                "1024",
                "-video_size",
                size,
                "-framerate",
                rate,
                "-pixel_format",
                "uyvy422",
            ]);
            let spec = format!("{video}:{}", audio.as_deref().unwrap_or("none"));
            push(&["-i", &spec]);
            ("[0:v]", audio.as_ref().map(|_| "[0:a]"))
        }
        #[cfg(test)]
        CaptureInput::Generated { video, audio } => {
            push(&["-re", "-f", "lavfi", "-i", video]);
            if *audio {
                push(&[
                    "-re",
                    "-f",
                    "lavfi",
                    "-i",
                    "sine=frequency=440:sample_rate=48000",
                ]);
            }
            ("[0:v]", audio.then_some("[1:a]"))
        }
    };

    // Vidéo : master intact, aperçu réduit, mesures 5 fois par seconde
    // (idet a besoin de toutes les images, avant la réduction de cadence).
    let mut graph = format!(
        "{video_ref}split=3[m][p][s];\
         [p]fps=12.5,scale={w}:-2,format=yuvj420p[pv];\
         [s]idet,fps=5,signalstats,metadata=mode=print,nullsink",
        w = config.preview_width
    );
    if let Some(audio_ref) = audio_ref {
        // Crêtes par canal toutes les 100 ms (4 800 échantillons à 48 kHz).
        graph.push_str(&format!(
            ";{audio_ref}asplit=2[am][al];\
             [al]asetnsamples=n=4800:p=0,\
             astats=metadata=1:reset=1:measure_perchannel=Peak_level:measure_overall=none,\
             ametadata=mode=print,anullsink"
        ));
    }
    push(&["-filter_complex", &graph]);

    push(&["-map", "[m]"]);
    if audio_ref.is_some() {
        push(&["-map", "[am]", "-c:a", "pcm_s16le"]);
    }
    push(&[
        "-c:v",
        "ffv1",
        "-level",
        "3",
        "-g",
        "1",
        "-slicecrc",
        "1",
        "-slices",
        "24",
        "-y",
    ]);
    a.push(config.master.clone().into_os_string());
    let mut push = |items: &[&str]| a.extend(items.iter().map(OsString::from));
    push(&[
        "-map",
        "[pv]",
        "-c:v",
        "mjpeg",
        "-q:v",
        "6",
        "-f",
        "image2pipe",
        "pipe:1",
    ]);
    a
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffmpeg::probe::probe;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn config(dir: &std::path::Path, video: &str, audio: bool) -> CaptureConfig {
        CaptureConfig {
            input: CaptureInput::Generated {
                video: video.into(),
                audio,
            },
            standard: TvStandard::Pal,
            master: dir.join("master.mkv"),
            preview_width: 384,
            end_of_tape_after_s: None,
            max_duration_s: None,
            min_free_bytes: 0,
        }
    }

    fn ffmpeg_here() -> bool {
        let ok = ffmpeg::ffmpeg().is_ok();
        if !ok {
            eprintln!("FFmpeg absent : lancer node scripts/fetch-ffmpeg.mjs");
        }
        ok
    }

    const MIRE: &str = "testsrc2=size=720x576:rate=25";

    #[test]
    fn directshow_arguments() {
        let mut c = config(std::path::Path::new("D:/Numeris/x"), MIRE, false);
        c.input = CaptureInput::DirectShow {
            video: "@device_pnp_video".into(),
            audio: Some("@device_cm_audio".into()),
            video_pin: Some(3),
        };
        let a: Vec<String> = args(&c)
            .into_iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        let joined = a.join(" ");
        assert!(joined.contains("-f dshow"));
        assert!(joined.contains("-video_size 720x576 -framerate 25"));
        assert!(joined.contains("-crossbar_video_input_pin_number 3"));
        assert!(a.contains(&"video=@device_pnp_video:audio=@device_cm_audio".to_owned()));
        assert!(joined.contains("-c:v ffv1 -level 3 -g 1 -slicecrc 1"));
        assert!(joined.contains("-map [am] -c:a pcm_s16le"));
        assert!(joined.ends_with("-f image2pipe pipe:1"));
    }

    #[test]
    fn stops_on_request_and_leaves_readable_master() {
        if !ffmpeg_here() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let previews = Arc::new(AtomicUsize::new(0));
        let telemetry = Arc::new(Mutex::new(Vec::new()));
        let (p, t) = (previews.clone(), telemetry.clone());
        let session = Session::start(
            config(dir.path(), MIRE, true),
            move |jpeg| {
                assert_eq!(&jpeg[..2], &[0xFF, 0xD8]);
                p.fetch_add(1, Ordering::Relaxed);
            },
            move |snap| t.lock().unwrap().push(snap.clone()),
        )
        .unwrap();
        std::thread::sleep(Duration::from_millis(2500));
        session.stop();
        let outcome = session.wait();

        assert_eq!(outcome.reason, StopReason::User);
        assert!(outcome.error.is_none(), "{:?}", outcome.error);
        assert!(previews.load(Ordering::Relaxed) >= 10);
        let last = outcome.last.unwrap();
        assert_eq!(last.signal, SignalState::Ok);
        // La sinusoïde de lavfi est mono : un seul canal.
        assert_eq!(last.peaks_db.len(), 1);
        // Amplitude par défaut de la sinusoïde : 1/8, soit −18 dBFS.
        assert!(
            (-19.0..-17.0).contains(&last.peaks_db[0]),
            "{:?}",
            last.peaks_db
        );
        assert!(!telemetry.lock().unwrap().is_empty());

        let info = probe(&dir.path().join("master.mkv")).unwrap();
        assert_eq!(info.video_codec.as_deref(), Some("ffv1"));
        assert_eq!(info.audio_codec.as_deref(), Some("pcm_s16le"));
        assert!(info.duration_ms.unwrap() >= 1500);

        let prints = finalize::fingerprints(&dir.path().join("master.mkv"), 2000, |_| {}).unwrap();
        assert_eq!(prints.blake3.len(), 64);
        let md5 = std::fs::read_to_string(prints.framemd5).unwrap();
        assert!(md5.lines().filter(|l| !l.starts_with('#')).count() > 30);
    }

    #[test]
    fn stops_at_end_of_tape_after_picture_then_black() {
        if !ffmpeg_here() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        // Deux secondes de noir (amorce, ignorée), deux d'image, puis du noir.
        let video = "color=black:size=720x576:rate=25:duration=2[a];\
                     testsrc2=size=720x576:rate=25:duration=2[b];\
                     color=black:size=720x576:rate=25[c];[a][b][c]concat=n=3";
        let mut c = config(dir.path(), video, false);
        c.end_of_tape_after_s = Some(1);
        c.max_duration_s = Some(20);
        let outcome = Session::start(c, |_| {}, |_| {}).unwrap().wait();
        assert_eq!(outcome.reason, StopReason::EndOfTape);
        let elapsed = outcome.last.unwrap().elapsed_ms;
        assert!((4500..9000).contains(&elapsed), "{elapsed}");
    }

    /// Essai sur vrai matériel, lancé à la main :
    /// `cargo test real_directshow_capture -- --ignored --nocapture`.
    /// Noms des périphériques par NUMERIS_TEST_VIDEO, NUMERIS_TEST_AUDIO et
    /// NUMERIS_TEST_PIN ; par défaut, le Dazzle DVC100 en S-Vidéo.
    #[test]
    #[ignore = "demande un boîtier de capture branché"]
    fn real_directshow_capture() {
        let env = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_owned());
        let dir = tempfile::tempdir().unwrap();
        let mut c = config(dir.path(), MIRE, false);
        c.input = CaptureInput::DirectShow {
            video: env("NUMERIS_TEST_VIDEO", "Dazzle DVC100 Video Device"),
            audio: Some(env(
                "NUMERIS_TEST_AUDIO",
                "Ligne (Dazzle DVC100 Audio Device)",
            )),
            video_pin: env("NUMERIS_TEST_PIN", "3").parse().ok(),
        };
        let session = Session::start(c, |_| {}, |t| eprintln!("{t:?}")).unwrap();
        std::thread::sleep(Duration::from_secs(6));
        session.stop();
        let outcome = session.wait();
        eprintln!("{outcome:?}");
        assert!(outcome.error.is_none());
        let last = outcome.last.unwrap();
        assert_eq!(last.dropped, 0, "images perdues");
        assert_eq!(last.peaks_db.len(), 2);
        let info = probe(&dir.path().join("master.mkv")).unwrap();
        eprintln!("{info:?}");
        assert_eq!((info.width, info.height), (Some(720), Some(576)));
        // Le boîtier met environ une seconde à démarrer.
        assert!(info.duration_ms.unwrap() >= 4000);
    }

    #[test]
    fn stops_when_disk_is_full() {
        if !ffmpeg_here() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let mut c = config(dir.path(), MIRE, false);
        c.min_free_bytes = u64::MAX;
        let outcome = Session::start(c, |_| {}, |_| {}).unwrap().wait();
        assert_eq!(outcome.reason, StopReason::DiskFull);
        assert!(probe(&dir.path().join("master.mkv")).is_ok());
    }
}
