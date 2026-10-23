//! FFmpeg (décision 0001) : ffmpeg et ffprobe sont des programmes séparés
//! livrés avec Numéris, appelés par le cœur. Jamais de liaison à leurs
//! bibliothèques : un plantage de FFmpeg ne fait pas tomber l'application.

pub mod devices;
pub mod probe;
pub mod proxy;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use crate::error::{Error, Result};

static DIR: OnceLock<PathBuf> = OnceLock::new();

/// Dossier de ffmpeg et ffprobe, donné au démarrage par l'application.
pub fn set_dir(path: PathBuf) {
    let _ = DIR.set(path);
}

fn tool(name: &str) -> Option<PathBuf> {
    let file = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    };
    DIR.get()
        .map(|d| d.join(&file))
        .filter(|p| p.is_file())
        .or_else(|| {
            // Tests et développement : la copie récupérée par scripts/fetch-ffmpeg.mjs.
            let dev = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("resources")
                .join("ffmpeg")
                .join(&file);
            dev.is_file().then_some(dev)
        })
}

/// Commande ffmpeg prête à recevoir ses arguments.
pub fn ffmpeg() -> Result<Command> {
    tool("ffmpeg")
        .map(|p| command(&p))
        .ok_or(Error::FfmpegMissing)
}

/// Commande ffprobe prête à recevoir ses arguments.
pub fn ffprobe() -> Result<Command> {
    tool("ffprobe")
        .map(|p| command(&p))
        .ok_or(Error::FfmpegMissing)
}

fn command(program: &Path) -> Command {
    let mut cmd = Command::new(program);
    cmd.stdin(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // Pas de fenêtre de console à chaque appel.
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

/// Version d'FFmpeg, telle que la donne `ffmpeg -version` (« 9.0.2 », « n9.0.2-17-g… »).
pub fn version() -> Result<String> {
    let output = ffmpeg()?.arg("-version").output()?;
    let text = String::from_utf8_lossy(&output.stdout);
    parse_version(&text).ok_or_else(|| Error::Tool {
        program: "ffmpeg",
        message: "version illisible".into(),
    })
}

fn parse_version(text: &str) -> Option<String> {
    text.lines()
        .next()?
        .strip_prefix("ffmpeg version ")?
        .split_whitespace()
        .next()
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_version_line() {
        let text = "ffmpeg version n9.0.2-17-g2a571b6068-20260930 Copyright (c) 2000-2026 the FFmpeg developers\nbuilt with gcc";
        assert_eq!(
            parse_version(text).as_deref(),
            Some("n9.0.2-17-g2a571b6068-20260930")
        );
        assert_eq!(parse_version("autre chose"), None);
    }

    #[test]
    fn bundled_ffmpeg_answers() {
        // La CI récupère FFmpeg avant les tests ; en local, le script doit avoir tourné.
        if tool("ffmpeg").is_none() {
            eprintln!("FFmpeg absent : lancer node scripts/fetch-ffmpeg.mjs");
            return;
        }
        assert!(version().unwrap().contains("9.0"));
    }
}
