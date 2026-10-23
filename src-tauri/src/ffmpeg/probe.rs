//! Informations d'un fichier vidéo lues par ffprobe, et ce que le lecteur de
//! l'interface sait lire directement.

use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub duration_ms: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    /// Image entrelacée (cas des cassettes analogiques et du DV).
    pub interlaced: bool,
    /// Le lecteur intégré (WebView2, WKWebView) lit-il le fichier tel quel ?
    pub playable: bool,
}

pub fn probe(path: &Path) -> Result<MediaInfo> {
    let output = super::ffprobe()?
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path)
        .output()?;
    if !output.status.success() {
        return Err(Error::Tool {
            program: "ffprobe",
            message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }
    let json: Value = serde_json::from_slice(&output.stdout)?;
    Ok(parse(&json))
}

fn parse(json: &Value) -> MediaInfo {
    let streams = json["streams"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let first = |kind: &str| streams.iter().find(|s| s["codec_type"] == kind);
    let video = first("video");
    let audio = first("audio");

    let video_codec = video
        .and_then(|v| v["codec_name"].as_str())
        .map(str::to_owned);
    let audio_codec = audio
        .and_then(|a| a["codec_name"].as_str())
        .map(str::to_owned);
    let interlaced = video
        .and_then(|v| v["field_order"].as_str())
        .is_some_and(|f| matches!(f, "tt" | "bb" | "tb" | "bt"));
    let container = json["format"]["format_name"].as_str().unwrap_or_default();

    let playable = video_codec.as_deref() == Some("h264")
        && !interlaced
        // ffprobe nomme ensemble les conteneurs de la famille MP4.
        && container.split(',').any(|c| c == "mp4" || c == "mov")
        && matches!(audio_codec.as_deref(), None | Some("aac" | "mp3"));

    let duration_ms = json["format"]["duration"]
        .as_str()
        .and_then(|d| d.parse::<f64>().ok())
        .map(|s| (s * 1000.0).round() as u64);

    MediaInfo {
        duration_ms,
        width: video.and_then(|v| v["width"].as_u64()).map(|w| w as u32),
        height: video.and_then(|v| v["height"].as_u64()).map(|h| h as u32),
        video_codec,
        audio_codec,
        interlaced,
        playable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn progressive_h264_mp4_is_playable() {
        let info = parse(&json!({
            "streams": [
                {"codec_type": "video", "codec_name": "h264", "width": 1920, "height": 1080, "field_order": "progressive"},
                {"codec_type": "audio", "codec_name": "aac"}
            ],
            "format": {"format_name": "mov,mp4,m4a,3gp,3g2,mj2", "duration": "12.345"}
        }));
        assert!(info.playable);
        assert!(!info.interlaced);
        assert_eq!(info.duration_ms, Some(12_345));
        assert_eq!((info.width, info.height), (Some(1920), Some(1080)));
    }

    #[test]
    fn dv_and_interlaced_files_need_a_proxy() {
        let dv = parse(&json!({
            "streams": [{"codec_type": "video", "codec_name": "dvvideo", "field_order": "bb"}],
            "format": {"format_name": "dv"}
        }));
        assert!(dv.interlaced);
        assert!(!dv.playable);
        let interlaced_h264 = parse(&json!({
            "streams": [{"codec_type": "video", "codec_name": "h264", "field_order": "tt"}],
            "format": {"format_name": "mov,mp4,m4a,3gp,3g2,mj2"}
        }));
        assert!(!interlaced_h264.playable);
    }
}
