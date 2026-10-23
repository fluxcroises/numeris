//! Bibliothèque : un dossier choisi par l'utilisateur, un sous-dossier par
//! cassette avec son `cassette.json`, et le catalogue SQLite dans `.numeris/`.
//! Les chemins enregistrés sont relatifs à la racine, pour qu'elle puisse
//! vivre sur un disque externe.

pub mod cassette;

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use crate::db;
use crate::error::{Error, Result};
use cassette::{CassetteFile, FORMAT_VERSION, TapeFormat, TvStandard, is_partial_date};

/// Dossier des fichiers internes de Numéris dans la bibliothèque.
pub const APP_DIR: &str = ".numeris";
const DB_FILE: &str = "catalogue.db";

pub struct Library {
    root: PathBuf,
    conn: Connection,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCassette {
    pub title: String,
    pub format: TapeFormat,
    pub tv_standard: Option<TvStandard>,
    pub recorded_from: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CassetteSummary {
    pub id: i64,
    pub uuid: String,
    pub folder: String,
    pub title: String,
    pub format: String,
    pub recorded_from: Option<String>,
    pub status: String,
    pub duration_ms: Option<i64>,
}

impl Library {
    /// Ouvre la bibliothèque, en créant son dossier et son catalogue au besoin.
    pub fn open(root: &Path) -> Result<Library> {
        let app_dir = root.join(APP_DIR);
        fs::create_dir_all(&app_dir)?;
        let conn = db::open(&app_dir.join(DB_FILE))?;
        Ok(Library {
            root: root.to_path_buf(),
            conn,
        })
    }

    /// Crée le dossier d'une cassette, son `cassette.json` et sa fiche au catalogue.
    pub fn create_cassette(&mut self, new: NewCassette) -> Result<CassetteSummary> {
        let title = new.title.trim().to_owned();
        if title.is_empty() {
            return Err(Error::Invalid("le titre est vide".into()));
        }
        let recorded_from = new
            .recorded_from
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        if let Some(date) = &recorded_from
            && !is_partial_date(date)
        {
            return Err(Error::Invalid(format!("date invalide : {date}")));
        }

        let folder = self.free_folder(&folder_name(&title, recorded_from.as_deref()))?;
        let dir = self.root.join(&folder);
        fs::create_dir(&dir)?;
        let file = CassetteFile {
            format_version: FORMAT_VERSION,
            uuid: uuid::Uuid::new_v4().to_string(),
            title,
            format: new.format,
            tv_standard: new.tv_standard,
            recorded_from,
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
        let inserted = file.write(&dir).and_then(|()| {
            self.conn.execute(
                "INSERT INTO cassettes (uuid, folder, title, format, tv_standard, recorded_from)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    file.uuid,
                    folder,
                    file.title,
                    file.format.as_str(),
                    file.tv_standard.map(TvStandard::as_str),
                    file.recorded_from,
                ],
            )?;
            Ok(())
        });
        if let Err(e) = inserted {
            // Le dossier vient d'être créé et ne contient que notre cassette.json.
            let _ = fs::remove_dir_all(&dir);
            return Err(e);
        }
        let id = self.conn.last_insert_rowid();
        self.cassette(id)
    }

    fn cassette(&self, id: i64) -> Result<CassetteSummary> {
        Ok(self.conn.query_row(
            &format!("{SUMMARY_SELECT} WHERE id = ?1"),
            [id],
            summary_row,
        )?)
    }

    /// Toutes les cassettes, de la plus ancienne date de tournage à la plus récente
    /// (les dates inconnues à la fin).
    pub fn cassettes(&self) -> Result<Vec<CassetteSummary>> {
        let mut stmt = self.conn.prepare(&format!(
            "{SUMMARY_SELECT} ORDER BY recorded_from IS NULL, recorded_from, title COLLATE NOCASE"
        ))?;
        let rows = stmt.query_map([], summary_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Nom de dossier libre, sur le disque comme au catalogue : « noel », « noel-2 »…
    fn free_folder(&self, base: &str) -> Result<String> {
        for n in 1.. {
            let name = if n == 1 {
                base.to_owned()
            } else {
                format!("{base}-{n}")
            };
            let taken: bool = self.conn.query_row(
                "SELECT EXISTS (SELECT 1 FROM cassettes WHERE folder = ?1)",
                [&name],
                |r| r.get(0),
            )?;
            if !taken && !self.root.join(&name).exists() {
                return Ok(name);
            }
        }
        unreachable!("la boucle ne se termine qu'en trouvant un nom libre")
    }
}

const SUMMARY_SELECT: &str =
    "SELECT id, uuid, folder, title, format, recorded_from, status, duration_ms FROM cassettes";

fn summary_row(row: &rusqlite::Row) -> rusqlite::Result<CassetteSummary> {
    Ok(CassetteSummary {
        id: row.get(0)?,
        uuid: row.get(1)?,
        folder: row.get(2)?,
        title: row.get(3)?,
        format: row.get(4)?,
        recorded_from: row.get(5)?,
        status: row.get(6)?,
        duration_ms: row.get(7)?,
    })
}

/// Nom de dossier lisible : « 1998-noel-chez-mamie ».
pub fn folder_name(title: &str, recorded_from: Option<&str>) -> String {
    let slug = slugify(title);
    let slug = if slug.is_empty() {
        "cassette".to_owned()
    } else {
        slug
    };
    match recorded_from.and_then(|d| d.get(..4)) {
        Some(year) => format!("{year}-{slug}"),
        None => slug,
    }
}

/// Minuscules ASCII sans accents, mots séparés par des tirets, 60 caractères au plus.
fn slugify(text: &str) -> String {
    let mut out = String::new();
    for c in text.nfd().filter(|c| c.is_ascii() || c.is_alphanumeric()) {
        match c {
            'a'..='z' | '0'..='9' => out.push(c),
            'A'..='Z' => out.push(c.to_ascii_lowercase()),
            // Lettres sans équivalent ASCII après décomposition (œ, æ, ß…) : ignorées.
            _ if !c.is_ascii() => {}
            _ => {
                if !out.is_empty() && !out.ends_with('-') {
                    out.push('-');
                }
            }
        }
        if out.len() >= 60 {
            break;
        }
    }
    out.trim_end_matches('-').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new(title: &str, date: Option<&str>) -> NewCassette {
        NewCassette {
            title: title.into(),
            format: TapeFormat::Vhs,
            tv_standard: Some(TvStandard::Pal),
            recorded_from: date.map(str::to_owned),
        }
    }

    #[test]
    fn folder_names() {
        assert_eq!(
            folder_name("Noël chez Mamie", Some("1998-12")),
            "1998-noel-chez-mamie"
        );
        assert_eq!(
            folder_name("  Les 10 ans de Léa !", None),
            "les-10-ans-de-lea"
        );
        assert_eq!(folder_name("???", None), "cassette");
        assert!(folder_name(&"a".repeat(200), None).len() <= 60);
    }

    #[test]
    fn creates_folder_json_and_catalogue_entry() {
        let dir = tempfile::tempdir().unwrap();
        let mut lib = Library::open(&dir.path().join("Numeris")).unwrap();
        let c = lib
            .create_cassette(new("Noël chez Mamie", Some("1998-12-24")))
            .unwrap();
        assert_eq!(c.folder, "1998-noel-chez-mamie");
        assert_eq!(c.status, "to_capture");
        let file = CassetteFile::read(&lib.root.join(&c.folder)).unwrap();
        assert_eq!(file.uuid, c.uuid);
        assert_eq!(file.title, "Noël chez Mamie");
        assert_eq!(lib.cassettes().unwrap(), vec![c]);
    }

    #[test]
    fn same_title_gets_its_own_folder() {
        let dir = tempfile::tempdir().unwrap();
        let mut lib = Library::open(dir.path()).unwrap();
        let a = lib.create_cassette(new("Vacances", Some("1994"))).unwrap();
        let b = lib.create_cassette(new("Vacances", Some("1994"))).unwrap();
        assert_eq!(a.folder, "1994-vacances");
        assert_eq!(b.folder, "1994-vacances-2");
    }

    #[test]
    fn rejects_empty_title_and_bad_date_without_leaving_files() {
        let dir = tempfile::tempdir().unwrap();
        let mut lib = Library::open(dir.path()).unwrap();
        assert!(lib.create_cassette(new("  ", None)).is_err());
        assert!(lib.create_cassette(new("Noël", Some("1998-13"))).is_err());
        let entries: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(entries, vec![std::ffi::OsString::from(APP_DIR)]);
    }

    #[test]
    fn sorts_by_date_with_unknown_last() {
        let dir = tempfile::tempdir().unwrap();
        let mut lib = Library::open(dir.path()).unwrap();
        lib.create_cassette(new("Sans date", None)).unwrap();
        lib.create_cassette(new("Mariage", Some("2001"))).unwrap();
        lib.create_cassette(new("Biarritz", Some("1994-08")))
            .unwrap();
        let titles: Vec<_> = lib
            .cassettes()
            .unwrap()
            .into_iter()
            .map(|c| c.title)
            .collect();
        assert_eq!(titles, ["Biarritz", "Mariage", "Sans date"]);
    }
}
