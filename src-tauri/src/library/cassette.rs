//! Fichier `cassette.json` : les métadonnées d'une cassette, à côté de son
//! master, lisibles sans Numéris. Le format est versionné ; toute évolution
//! prévoit une migration.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

pub const CASSETTE_FILE: &str = "cassette.json";
pub const FORMAT_VERSION: u32 = 1;

/// Support d'origine (mêmes valeurs que la colonne `cassettes.format`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TapeFormat {
    Vhs,
    VhsC,
    Svhs,
    Video8,
    Hi8,
    Digital8,
    Minidv,
    /// Fichier déjà numérisé, importé.
    File,
}

impl TapeFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            TapeFormat::Vhs => "vhs",
            TapeFormat::VhsC => "vhs_c",
            TapeFormat::Svhs => "svhs",
            TapeFormat::Video8 => "video8",
            TapeFormat::Hi8 => "hi8",
            TapeFormat::Digital8 => "digital8",
            TapeFormat::Minidv => "minidv",
            TapeFormat::File => "file",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TvStandard {
    Pal,
    Secam,
    Ntsc,
}

impl TvStandard {
    pub fn as_str(self) -> &'static str {
        match self {
            TvStandard::Pal => "pal",
            TvStandard::Secam => "secam",
            TvStandard::Ntsc => "ntsc",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MasterFile {
    /// Nom du fichier dans le dossier de la cassette (`master.mkv`, `master.dv`).
    pub file: String,
    pub blake3: Option<String>,
    pub framemd5: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterFile {
    pub position: u32,
    pub start_ms: u64,
    pub end_ms: u64,
    #[serde(default)]
    pub title: String,
    pub recorded_at: Option<String>,
    pub place: Option<String>,
    #[serde(default)]
    pub people: Vec<String>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CassetteFile {
    pub format_version: u32,
    pub uuid: String,
    pub title: String,
    pub format: TapeFormat,
    pub tv_standard: Option<TvStandard>,
    /// Date de tournage, éventuellement imprécise : « 1998 », « 1998-12 », « 1998-12-24 ».
    pub recorded_from: Option<String>,
    pub recorded_to: Option<String>,
    pub place: Option<String>,
    #[serde(default)]
    pub people: Vec<String>,
    #[serde(default)]
    pub notes: String,
    pub label_photo: Option<String>,
    pub master: Option<MasterFile>,
    pub proxy: Option<String>,
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub chapters: Vec<ChapterFile>,
}

impl CassetteFile {
    // Servira au MVP à reconstruire le catalogue depuis les dossiers.
    #[cfg(test)]
    pub fn read(dir: &Path) -> Result<CassetteFile> {
        Ok(serde_json::from_slice(&fs::read(dir.join(CASSETTE_FILE))?)?)
    }

    /// Écrit le fichier d'un coup : un fichier temporaire est renommé à la fin,
    /// pour ne jamais laisser un `cassette.json` à moitié écrit.
    pub fn write(&self, dir: &Path) -> Result<()> {
        let tmp = dir.join("cassette.json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        fs::rename(&tmp, dir.join(CASSETTE_FILE))?;
        Ok(())
    }
}

/// Date imprécise valide : « AAAA », « AAAA-MM » ou « AAAA-MM-JJ ».
pub fn is_partial_date(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    let number = |p: &str, len: usize| {
        (p.len() == len && p.bytes().all(|b| b.is_ascii_digit()))
            .then(|| p.parse::<u32>().ok())
            .flatten()
    };
    let Some(year) = parts.first().and_then(|p| number(p, 4)) else {
        return false;
    };
    if !(1900..=2100).contains(&year) {
        return false;
    }
    match parts.len() {
        1 => true,
        2 => matches!(number(parts[1], 2), Some(1..=12)),
        3 => {
            let (Some(month @ 1..=12), Some(day)) = (number(parts[1], 2), number(parts[2], 2))
            else {
                return false;
            };
            let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
            let days = match month {
                2 if leap => 29,
                2 => 28,
                4 | 6 | 9 | 11 => 30,
                _ => 31,
            };
            (1..=days).contains(&day)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_dates() {
        for ok in ["1998", "1998-12", "1998-12-24", "2000-02-29"] {
            assert!(is_partial_date(ok), "{ok}");
        }
        for bad in [
            "98",
            "1998-13",
            "1998-1",
            "1999-02-29",
            "1998-12-32",
            "noël",
            "1850",
        ] {
            assert!(!is_partial_date(bad), "{bad}");
        }
    }

    #[test]
    fn json_uses_readable_names() {
        let file = CassetteFile {
            format_version: FORMAT_VERSION,
            uuid: "u".into(),
            title: "Noël".into(),
            format: TapeFormat::VhsC,
            tv_standard: Some(TvStandard::Pal),
            recorded_from: Some("1998-12".into()),
            recorded_to: None,
            place: None,
            people: vec![],
            notes: String::new(),
            label_photo: None,
            master: None,
            proxy: None,
            duration_ms: None,
            chapters: vec![],
        };
        let json = serde_json::to_string(&file).unwrap();
        assert!(json.contains("\"formatVersion\":1"));
        assert!(json.contains("\"format\":\"vhs_c\""));
        assert!(json.contains("\"tvStandard\":\"pal\""));
        let dir = tempfile::tempdir().unwrap();
        file.write(dir.path()).unwrap();
        assert_eq!(CassetteFile::read(dir.path()).unwrap(), file);
    }
}
