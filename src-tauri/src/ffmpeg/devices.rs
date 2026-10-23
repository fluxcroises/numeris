//! Périphériques de capture vus par FFmpeg : DirectShow sous Windows,
//! AVFoundation sous macOS. FFmpeg écrit la liste sur sa sortie d'erreur et
//! se termine en échec (il n'a rien à capturer) : seul le texte compte.

use serde::Serialize;

use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DeviceKind {
    Video,
    Audio,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDevice {
    /// Nom affiché à l'utilisateur.
    pub name: String,
    /// Ce qu'il faut donner à FFmpeg pour l'ouvrir : nom unique DirectShow
    /// (« Alternative name ») ou indice AVFoundation.
    pub id: String,
    pub kind: DeviceKind,
    /// Caméscope DV relié en FireWire (pilote « Microsoft DV Camera and VCR »).
    pub dv: bool,
}

/// Liste les périphériques de capture branchés.
pub fn list() -> Result<Vec<CaptureDevice>> {
    let mut cmd = super::ffmpeg()?;
    cmd.args(["-hide_banner", "-list_devices", "true"]);
    if cfg!(target_os = "macos") {
        cmd.args(["-f", "avfoundation", "-i", ""]);
    } else {
        cmd.args(["-f", "dshow", "-i", "dummy"]);
    }
    let output = cmd.output()?;
    let text = String::from_utf8_lossy(&output.stderr);
    Ok(if cfg!(target_os = "macos") {
        parse_avfoundation(&text)
    } else {
        parse_dshow(&text)
    })
}

/// Retire le préfixe « [in#0 @ 000002e0f1e40ac0] » d'une ligne de journal.
fn message(line: &str) -> &str {
    let line = line.trim();
    match line.strip_prefix('[').and_then(|r| r.split_once("] ")) {
        Some((_, rest)) => rest,
        None => line,
    }
}

fn is_dv(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("dv camera") || lower.contains("dv vcr") || lower.contains("1394")
}

/// `"UGREEN Camera" (video)` puis, ligne suivante, `Alternative name "@device_…"`.
pub(crate) fn parse_dshow(text: &str) -> Vec<CaptureDevice> {
    let mut devices: Vec<CaptureDevice> = Vec::new();
    // Indices des entrées du dernier périphérique lu, pour y rattacher son nom unique.
    let mut last: Vec<usize> = Vec::new();
    for line in text.lines() {
        let msg = message(line);
        if let Some(rest) = msg.trim_start().strip_prefix("Alternative name ") {
            let id = rest.trim().trim_matches('"').to_owned();
            for &i in &last {
                devices[i].id = id.clone();
            }
            last.clear();
            continue;
        }
        let Some(rest) = msg.strip_prefix('"') else {
            continue;
        };
        let Some((name, kinds)) = rest.split_once("\" (") else {
            continue;
        };
        last.clear();
        // « (video) », « (audio) » ou « (audio, video) » ; « (none) » est ignoré.
        for kind in kinds.trim_end_matches(')').split(',').map(str::trim) {
            let kind = match kind {
                "video" => DeviceKind::Video,
                "audio" => DeviceKind::Audio,
                _ => continue,
            };
            last.push(devices.len());
            devices.push(CaptureDevice {
                name: name.to_owned(),
                id: name.to_owned(),
                kind,
                dv: kind == DeviceKind::Video && is_dv(name),
            });
        }
    }
    devices
}

/// « AVFoundation video devices: », puis « [0] FaceTime HD Camera »…
pub(crate) fn parse_avfoundation(text: &str) -> Vec<CaptureDevice> {
    let mut devices = Vec::new();
    let mut kind = None;
    for line in text.lines() {
        let msg = message(line);
        if msg.contains("AVFoundation video devices") {
            kind = Some(DeviceKind::Video);
            continue;
        }
        if msg.contains("AVFoundation audio devices") {
            kind = Some(DeviceKind::Audio);
            continue;
        }
        let Some(kind) = kind else { continue };
        let Some((index, name)) = msg.strip_prefix('[').and_then(|r| r.split_once("] ")) else {
            continue;
        };
        if index.parse::<u32>().is_err() {
            continue;
        }
        // Les captures d'écran ne sont pas des sources vidéo pour Numéris.
        if kind == DeviceKind::Video && name.starts_with("Capture screen") {
            continue;
        }
        devices.push(CaptureDevice {
            name: name.to_owned(),
            id: index.to_owned(),
            kind,
            dv: kind == DeviceKind::Video && is_dv(name),
        });
    }
    devices
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sortie relevée sous Windows 11 avec FFmpeg 9.0.2 (webcam USB et webcam logicielle).
    const DSHOW: &str = r#"[in#0 @ 000002e0f1e40ac0] "UGREEN Camera" (video)
[in#0 @ 000002e0f1e40ac0]   Alternative name "@device_pnp_\\?\usb#vid_0c45&pid_2283&mi_00#6&287ca144&0&0000#{65e8773d-8f56-11d0-a3b9-00a0c9223196}\global"
[in#0 @ 000002e0f1e40ac0] "FUJIFILM X Webcam 2" (video)
[in#0 @ 000002e0f1e40ac0]   Alternative name "@device_sw_{860BB310-5D01-11D0-BD3B-00A0C911CE86}\{8DB752C3-30F4-4E10-A3B1-AB12A9E94B21}"
[in#0 @ 000002e0f1e40ac0] "Microsoft DV Camera and VCR" (video)
[in#0 @ 000002e0f1e40ac0]   Alternative name "@device_pnp_dv"
[in#0 @ 000002e0f1e40ac0] "Microphone (UGREEN Camera)" (audio)
[in#0 @ 000002e0f1e40ac0]   Alternative name "@device_cm_{33D9A762-90C8-11D0-BD43-00A0C911CE86}\wave_{5E61CBF1-D1F6-43BC-A2AF-38B1F2BE51E5}"
Error opening input file dummy."#;

    #[test]
    fn parses_dshow() {
        let devices = parse_dshow(DSHOW);
        assert_eq!(devices.len(), 4);
        assert_eq!(devices[0].name, "UGREEN Camera");
        assert_eq!(devices[0].kind, DeviceKind::Video);
        assert!(devices[0].id.starts_with("@device_pnp_"));
        assert!(!devices[0].dv);
        assert!(devices[2].dv);
        assert_eq!(devices[3].name, "Microphone (UGREEN Camera)");
        assert_eq!(devices[3].kind, DeviceKind::Audio);
    }

    #[test]
    fn dshow_device_with_audio_and_video() {
        let text = "[in#0 @ 0] \"Boîtier\" (audio, video)\n[in#0 @ 0]   Alternative name \"@x\"";
        let devices = parse_dshow(text);
        assert_eq!(devices.len(), 2);
        assert!(devices.iter().all(|d| d.id == "@x"));
    }

    #[test]
    fn parses_avfoundation() {
        let text = "[AVFoundation indev @ 0x1] AVFoundation video devices:
[AVFoundation indev @ 0x1] [0] FaceTime HD Camera
[AVFoundation indev @ 0x1] [1] USB Video
[AVFoundation indev @ 0x1] [2] Capture screen 0
[AVFoundation indev @ 0x1] AVFoundation audio devices:
[AVFoundation indev @ 0x1] [0] USB Audio
[in#0 @ 0x2] Error opening input: Input/output error";
        let devices = parse_avfoundation(text);
        assert_eq!(devices.len(), 3);
        assert_eq!(devices[1].name, "USB Video");
        assert_eq!(devices[1].id, "1");
        assert_eq!(devices[2].kind, DeviceKind::Audio);
        assert_eq!(devices[2].id, "0");
    }
}
