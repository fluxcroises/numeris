//! Copie de lecture (proxy) : H.264 désentrelacé, 576 lignes au plus, images
//! clés rapprochées pour naviguer précisément. Le lecteur intégré ne lit ni
//! FFV1, ni DV, ni vidéo entrelacée ; l'original n'est qu'ouvert en lecture.

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Stdio;

use crate::error::{Error, Result};

use super::probe::MediaInfo;

/// Écrit le proxy de `source` dans `out`, par un fichier temporaire renommé à la
/// fin. `progress` reçoit l'avancement entre 0 et 1.
pub fn make(
    source: &Path,
    info: &MediaInfo,
    out: &Path,
    mut progress: impl FnMut(f64),
) -> Result<()> {
    let tmp = out.with_extension("tmp.mp4");
    let mut filters = Vec::new();
    if info.interlaced {
        filters.push("bwdif=mode=send_frame");
    }
    filters.push("scale=-2:'min(576,ih)'");
    filters.push("format=yuv420p");

    let mut child = super::ffmpeg()?
        .args(["-hide_banner", "-nostats", "-loglevel", "error", "-y", "-i"])
        .arg(source)
        .args(["-map", "0:v:0", "-map", "0:a:0?", "-vf"])
        .arg(filters.join(","))
        .args([
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            "21",
            // Une image clé par seconde : navigation précise dans le lecteur.
            "-g",
            "25",
            "-c:a",
            "aac",
            "-b:a",
            "160k",
            "-movflags",
            "+faststart",
            "-progress",
            "pipe:1",
        ])
        .arg(&tmp)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let duration_us = info.duration_ms.map(|ms| ms as f64 * 1000.0);
    if let Some(stdout) = child.stdout.take() {
        for line in BufReader::new(stdout)
            .lines()
            .map_while(std::io::Result::ok)
        {
            if let (Some(value), Some(total)) = (line.strip_prefix("out_time_us="), duration_us)
                && let Ok(done) = value.parse::<f64>()
                && total > 0.0
            {
                progress((done / total).clamp(0.0, 1.0));
            }
        }
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        let _ = std::fs::remove_file(&tmp);
        return Err(Error::Tool {
            program: "ffmpeg",
            message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }
    std::fs::rename(&tmp, out)?;
    progress(1.0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::probe::probe;
    use super::*;

    #[test]
    fn interlaced_ffv1_becomes_playable_proxy() {
        let Ok(mut cmd) = super::super::ffmpeg() else {
            eprintln!("FFmpeg absent : lancer node scripts/fetch-ffmpeg.mjs");
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        // Échantillon généré par FFmpeg lui-même : mire PAL entrelacée, comme un master.
        let master = dir.path().join("master.mkv");
        let status = cmd
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc2=size=720x576:rate=25:duration=2",
            ])
            .args(["-f", "lavfi", "-i", "sine=frequency=440:duration=2"])
            .args(["-vf", "setfield=tff", "-c:v", "ffv1", "-field_order", "tt"])
            .args(["-c:a", "pcm_s16le"])
            .arg(&master)
            .status()
            .unwrap();
        assert!(status.success());

        let info = probe(&master).unwrap();
        assert!(info.interlaced);
        assert!(!info.playable);
        assert_eq!(info.duration_ms.map(|d| d / 100), Some(20));

        let proxy = dir.path().join("proxy.mp4");
        let mut last = 0.0;
        make(&master, &info, &proxy, |p| last = p).unwrap();
        assert_eq!(last, 1.0);
        let out = probe(&proxy).unwrap();
        assert!(out.playable, "{out:?}");
        assert_eq!(out.height, Some(576));
        assert!(!dir.path().join("proxy.tmp.mp4").exists());
    }
}
