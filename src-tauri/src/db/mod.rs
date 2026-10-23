//! Base SQLite du catalogue : ouverture, réglages de connexion et migrations
//! numérotées.
//!
//! Chaque migration de `migrations/` est appliquée une seule fois, dans une
//! transaction ; la version courante est suivie par `PRAGMA user_version`.

use std::path::Path;
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension};

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("erreur SQLite : {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error(
        "la base est en version {found}, plus récente que cette version de Numéris ({supported})"
    )]
    TooNew { found: u32, supported: u32 },
    #[error("la migration {0} laisserait des liens rompus dans la table {1}")]
    ForeignKeys(u32, String),
}

/// Migrations dans l'ordre ; l'indice + 1 est le numéro de version.
const MIGRATIONS: &[&str] = &[include_str!("migrations/0001_initial.sql")];

pub const SCHEMA_VERSION: u32 = MIGRATIONS.len() as u32;

/// Ouvre (ou crée) la base et applique les migrations manquantes.
pub fn open(path: &Path) -> Result<Connection, DbError> {
    let mut conn = Connection::open(path)?;
    configure(&conn)?;
    migrate(&mut conn)?;
    Ok(conn)
}

/// Base en mémoire, pour les tests.
#[cfg(test)]
pub fn open_in_memory() -> Result<Connection, DbError> {
    let mut conn = Connection::open_in_memory()?;
    configure(&conn)?;
    migrate(&mut conn)?;
    Ok(conn)
}

fn configure(conn: &Connection) -> Result<(), DbError> {
    conn.busy_timeout(Duration::from_secs(5))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", true)?;
    Ok(())
}

fn migrate(conn: &mut Connection) -> Result<(), DbError> {
    let current: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if current > SCHEMA_VERSION {
        return Err(DbError::TooNew {
            found: current,
            supported: SCHEMA_VERSION,
        });
    }
    if current == SCHEMA_VERSION {
        return Ok(());
    }
    // Une table recréée pour changer ses contraintes ne doit pas déclencher
    // les actions des clés étrangères : elles sont coupées le temps des
    // migrations, hors transaction, et vérifiées avant chaque validation.
    conn.pragma_update(None, "foreign_keys", false)?;
    let result = (|| -> Result<(), DbError> {
        for (index, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            let broken: Option<String> = tx
                .query_row(
                    "SELECT \"table\" FROM pragma_foreign_key_check LIMIT 1",
                    [],
                    |r| r.get(0),
                )
                .optional()?;
            if let Some(table) = broken {
                return Err(DbError::ForeignKeys(index as u32 + 1, table));
            }
            tx.pragma_update(None, "user_version", index as u32 + 1)?;
            tx.commit()?;
        }
        Ok(())
    })();
    conn.pragma_update(None, "foreign_keys", true)?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(conn: &Connection) -> u32 {
        conn.pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn migrates_to_latest_version() {
        let conn = open_in_memory().unwrap();
        assert_eq!(version(&conn), SCHEMA_VERSION);
        let fk: bool = conn
            .pragma_query_value(None, "foreign_keys", |r| r.get(0))
            .unwrap();
        assert!(fk);
    }

    #[test]
    fn reopening_keeps_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("numeris.db");
        {
            let conn = open(&path).unwrap();
            conn.execute(
                "INSERT INTO cassettes (uuid, folder, title, format) VALUES ('u1', 'noel', 'Noël', 'vhs')",
                [],
            )
            .unwrap();
        }
        let conn = open(&path).unwrap();
        let title: String = conn
            .query_row("SELECT title FROM cassettes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(title, "Noël");
    }

    #[test]
    fn refuses_newer_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("numeris.db");
        Connection::open(&path)
            .unwrap()
            .pragma_update(None, "user_version", SCHEMA_VERSION + 1)
            .unwrap();
        assert!(matches!(open(&path), Err(DbError::TooNew { .. })));
    }

    #[test]
    fn chapters_follow_their_cassette() {
        let conn = open_in_memory().unwrap();
        conn.execute_batch(
            "INSERT INTO cassettes (id, uuid, folder, title, format) VALUES (1, 'u1', 'noel', 'Noël', 'vhs');
             INSERT INTO chapters (cassette_id, position, start_ms, end_ms) VALUES (1, 1, 0, 1000);
             DELETE FROM cassettes WHERE id = 1;",
        )
        .unwrap();
        let count: i64 = conn
            .query_row("SELECT count(*) FROM chapters", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn rejects_unknown_format_and_bad_chapter() {
        let conn = open_in_memory().unwrap();
        assert!(
            conn.execute(
                "INSERT INTO cassettes (uuid, folder, title, format) VALUES ('u', 'f', 't', 'betamax')",
                [],
            )
            .is_err()
        );
        conn.execute(
            "INSERT INTO cassettes (id, uuid, folder, title, format) VALUES (1, 'u1', 'f', 't', 'hi8')",
            [],
        )
        .unwrap();
        assert!(
            conn.execute(
                "INSERT INTO chapters (cassette_id, position, start_ms, end_ms) VALUES (1, 1, 500, 500)",
                [],
            )
            .is_err()
        );
    }
}
