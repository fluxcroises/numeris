//! Empreintes du master : BLAKE3 du fichier entier (vérification rapide) et
//! framemd5 (une empreinte par image décodée, pour localiser une corruption).
//! Un master n'est considéré comme écrit qu'une fois les deux enregistrés.

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Stdio;

use crate::error::{Error, Result};
use crate::ffmpeg;

pub struct Fingerprints {
    pub blake3: String,
    pub framemd5: PathBuf,
}

/// BLAKE3 du fichier, lu par blocs ; `progress` reçoit l'avancement de 0 à 1.
pub fn blake3_file(path: &Path, mut progress: impl FnMut(f64)) -> Result<String> {
    let total = std::fs::metadata(path)?.len().max(1);
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0; 4 << 20];
    let mut done = 0u64;
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        done += n as u64;
        progress(done as f64 / total as f64);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

/// Écrit `master.framemd5` à côté du master (fichier temporaire renommé à la fin).
pub fn framemd5(master: &Path, duration_ms: u64, mut progress: impl FnMut(f64)) -> Result<PathBuf> {
    let out = master.with_extension("framemd5");
    let tmp = master.with_extension("framemd5.tmp");
    let mut child = ffmpeg::ffmpeg()?
        .args(["-hide_banner", "-nostats", "-loglevel", "error", "-y", "-i"])
        .arg(master)
        .args(["-map", "0", "-f", "framemd5", "-progress", "pipe:1"])
        .arg(&tmp)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(stdout) = child.stdout.take() {
        for line in BufReader::new(stdout)
            .lines()
            .map_while(std::io::Result::ok)
        {
            if let Some(us) = line
                .strip_prefix("out_time_us=")
                .and_then(|v| v.parse::<u64>().ok())
                && duration_ms > 0
            {
                progress((us as f64 / 1000.0 / duration_ms as f64).clamp(0.0, 1.0));
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
    std::fs::rename(&tmp, &out)?;
    Ok(out)
}

/// Les deux empreintes ; l'avancement va de 0 à 0,5 pour BLAKE3, puis à 1.
pub fn fingerprints(
    master: &Path,
    duration_ms: u64,
    mut progress: impl FnMut(f64),
) -> Result<Fingerprints> {
    let blake3 = blake3_file(master, |p| progress(p * 0.5))?;
    let framemd5 = framemd5(master, duration_ms, |p| progress(0.5 + p * 0.5))?;
    progress(1.0);
    Ok(Fingerprints { blake3, framemd5 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blake3_matches_reference() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.bin");
        std::fs::write(&path, b"Numeris").unwrap();
        let mut last = 0.0;
        let hash = blake3_file(&path, |p| last = p).unwrap();
        assert_eq!(hash, blake3::hash(b"Numeris").to_hex().to_string());
        assert_eq!(last, 1.0);
    }
}
