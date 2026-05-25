use crate::error::AppResult;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

const MIGRATIONS: &[Migration] = &[
    Migration {
        id: 1,
        name: "create_app_metadata",
        sql: r#"
            CREATE TABLE IF NOT EXISTS app_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
        "#,
    },
    Migration {
        id: 2,
        name: "create_transcript_history",
        sql: r#"
            CREATE TABLE IF NOT EXISTS transcript_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                raw_text TEXT NOT NULL,
                final_text TEXT NOT NULL,
                output_style TEXT NOT NULL,
                recognition_started_at INTEGER NOT NULL,
                recognition_duration_ms INTEGER NOT NULL,
                injected INTEGER NOT NULL,
                error_code TEXT,
                error_summary TEXT,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_transcript_history_created_at
                ON transcript_history(created_at);
        "#,
    },
    Migration {
        id: 3,
        name: "remove_failed_transcript_history",
        sql: r#"
            DELETE FROM transcript_history
            WHERE COALESCE(TRIM(error_code), '') <> ''
               OR COALESCE(TRIM(error_summary), '') <> '';
        "#,
    },
];

#[derive(Debug)]
struct Migration {
    id: i64,
    name: &'static str,
    sql: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseHealth {
    pub ok: bool,
    pub path: String,
    pub applied_migrations: i64,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TranscriptHistoryRecord {
    pub id: i64,
    pub raw_text: String,
    pub final_text: String,
    pub output_style: String,
    pub recognition_started_at: i64,
    pub recognition_duration_ms: i64,
    pub injected: bool,
    pub error_code: Option<String>,
    pub error_summary: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TranscriptHistoryPage {
    pub items: Vec<TranscriptHistoryRecord>,
    pub total: i64,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug)]
pub struct Database {
    path: PathBuf,
    connection: Mutex<Connection>,
}

#[derive(Debug, Clone)]
pub struct NewTranscriptHistory<'a> {
    pub raw_text: &'a str,
    pub final_text: &'a str,
    pub output_style: &'a str,
    pub recognition_started_at: &'a str,
    pub recognition_duration_ms: i64,
    pub injected: bool,
    pub error_code: Option<&'a str>,
    pub error_summary: Option<&'a str>,
}

impl Database {
    pub fn open(path: impl Into<PathBuf>) -> AppResult<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let connection = Connection::open(&path)?;
        let database = Self {
            path,
            connection: Mutex::new(connection),
        };
        database.apply_migrations()?;
        database.set_metadata("schema_ready", "true")?;
        Ok(database)
    }

    pub fn apply_migrations(&self) -> AppResult<()> {
        let mut connection = self.connection.lock().map_err(|_| crate::error::AppError::StateLock)?;
        connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS migration_history (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                applied_at TEXT NOT NULL
            );
            "#,
        )?;

        let transaction = connection.transaction()?;
        for migration in MIGRATIONS {
            let already_applied: Option<i64> = transaction
                .query_row(
                    "SELECT id FROM migration_history WHERE id = ?1",
                    params![migration.id],
                    |row| row.get(0),
                )
                .optional()?;

            if already_applied.is_none() {
                transaction.execute_batch(migration.sql)?;
                transaction.execute(
                    "INSERT INTO migration_history (id, name, applied_at) VALUES (?1, ?2, ?3)",
                    params![migration.id, migration.name, now_string()],
                )?;
            }
        }

        transaction.commit()?;
        Ok(())
    }

    pub fn health(&self) -> DatabaseHealth {
        match (self.count_migrations(), self.get_metadata("schema_ready")) {
            (Ok(applied_migrations), Ok(Some(schema_ready))) if schema_ready == "true" => DatabaseHealth {
                ok: true,
                path: self.path.display().to_string(),
                applied_migrations,
                last_error: None,
            },
            (Ok(applied_migrations), Ok(_)) => DatabaseHealth {
                ok: false,
                path: self.path.display().to_string(),
                applied_migrations,
                last_error: Some("database metadata is not initialized".to_string()),
            },
            (Err(error), _) | (_, Err(error)) => DatabaseHealth {
                ok: false,
                path: self.path.display().to_string(),
                applied_migrations: 0,
                last_error: Some(error.to_string()),
            },
        }
    }

    pub fn count_migrations(&self) -> AppResult<i64> {
        let connection = self.connection.lock().map_err(|_| crate::error::AppError::StateLock)?;
        let count = connection.query_row("SELECT COUNT(*) FROM migration_history", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn set_metadata(&self, key: &str, value: &str) -> AppResult<()> {
        let connection = self.connection.lock().map_err(|_| crate::error::AppError::StateLock)?;
        connection.execute(
            r#"
            INSERT INTO app_metadata (key, value, updated_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(key)
            DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
            "#,
            params![key, value, now_string()],
        )?;
        Ok(())
    }

    pub fn get_metadata(&self, key: &str) -> AppResult<Option<String>> {
        let connection = self.connection.lock().map_err(|_| crate::error::AppError::StateLock)?;
        let value = connection
            .query_row("SELECT value FROM app_metadata WHERE key = ?1", params![key], |row| row.get(0))
            .optional()?;
        Ok(value)
    }

    pub fn insert_transcript_history(&self, entry: NewTranscriptHistory<'_>) -> AppResult<i64> {
        let connection = self.connection.lock().map_err(|_| crate::error::AppError::StateLock)?;
        connection.execute(
            r#"
            INSERT INTO transcript_history (
                raw_text,
                final_text,
                output_style,
                recognition_started_at,
                recognition_duration_ms,
                injected,
                error_code,
                error_summary,
                created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                entry.raw_text,
                entry.final_text,
                entry.output_style,
                entry.recognition_started_at.parse::<i64>().unwrap_or_default(),
                entry.recognition_duration_ms,
                if entry.injected { 1 } else { 0 },
                entry.error_code,
                entry.error_summary,
                now_i64(),
            ],
        )?;
        Ok(connection.last_insert_rowid())
    }

    pub fn clear_transcript_history(&self) -> AppResult<usize> {
        let connection = self.connection.lock().map_err(|_| crate::error::AppError::StateLock)?;
        let deleted = connection.execute("DELETE FROM transcript_history", [])?;
        Ok(deleted)
    }

    pub fn delete_transcript_history_item(&self, id: i64) -> AppResult<usize> {
        let connection = self.connection.lock().map_err(|_| crate::error::AppError::StateLock)?;
        let deleted = connection.execute("DELETE FROM transcript_history WHERE id = ?1", params![id])?;
        Ok(deleted)
    }

    pub fn get_transcript_history(&self, limit: u32, offset: u32) -> AppResult<TranscriptHistoryPage> {
        let connection = self.connection.lock().map_err(|_| crate::error::AppError::StateLock)?;
        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                raw_text,
                final_text,
                output_style,
                recognition_started_at,
                recognition_duration_ms,
                injected,
                error_code,
                error_summary,
                created_at
            FROM transcript_history
            WHERE COALESCE(TRIM(error_code), '') = ''
              AND COALESCE(TRIM(error_summary), '') = ''
            ORDER BY created_at DESC, id DESC
            LIMIT ?1 OFFSET ?2
            "#,
        )?;
        let rows = statement.query_map(params![i64::from(limit), i64::from(offset)], |row| {
            Ok(TranscriptHistoryRecord {
                id: row.get(0)?,
                raw_text: row.get(1)?,
                final_text: row.get(2)?,
                output_style: row.get(3)?,
                recognition_started_at: row.get(4)?,
                recognition_duration_ms: row.get(5)?,
                injected: row.get::<_, i64>(6)? != 0,
                error_code: row.get(7)?,
                error_summary: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        let items = rows.collect::<Result<Vec<_>, _>>()?;
        let total = connection.query_row(
            r#"
            SELECT COUNT(*)
            FROM transcript_history
            WHERE COALESCE(TRIM(error_code), '') = ''
              AND COALESCE(TRIM(error_summary), '') = ''
            "#,
            [],
            |row| row.get(0),
        )?;
        Ok(TranscriptHistoryPage {
            items,
            total,
            limit,
            offset,
        })
    }

    pub fn cleanup_transcript_history(&self, retention_days: u16) -> AppResult<usize> {
        let retention_seconds = i64::from(retention_days.max(1)) * 24 * 60 * 60;
        let cutoff = now_i64().saturating_sub(retention_seconds);
        let connection = self.connection.lock().map_err(|_| crate::error::AppError::StateLock)?;
        let deleted = connection.execute(
            "DELETE FROM transcript_history WHERE created_at < ?1",
            params![cutoff],
        )?;
        Ok(deleted)
    }

    #[cfg(test)]
    pub fn count_transcript_history(&self) -> AppResult<i64> {
        let connection = self.connection.lock().map_err(|_| crate::error::AppError::StateLock)?;
        let count = connection.query_row("SELECT COUNT(*) FROM transcript_history", [], |row| row.get(0))?;
        Ok(count)
    }
}

fn now_string() -> String {
    now_i64().to_string()
}

fn now_i64() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn test_db_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("flowtype-db-{name}-{unique}")).join("app.db")
    }

    #[test]
    fn open_creates_database_and_applies_migrations() {
        let path = test_db_path("open");

        let database = Database::open(&path).unwrap();
        let health = database.health();

        assert!(path.exists());
        assert!(health.ok);
        assert_eq!(health.applied_migrations, 3);
    }

    #[test]
    fn migrations_are_idempotent() {
        let path = test_db_path("idempotent");
        let database = Database::open(&path).unwrap();

        database.apply_migrations().unwrap();
        database.apply_migrations().unwrap();

        assert_eq!(database.count_migrations().unwrap(), 3);
    }

    #[test]
    fn metadata_dao_can_write_and_read_values() {
        let path = test_db_path("metadata");
        let database = Database::open(&path).unwrap();

        database.set_metadata("schema_ready", "true").unwrap();
        database.set_metadata("schema_ready", "still_true").unwrap();

        assert_eq!(
            database.get_metadata("schema_ready").unwrap(),
            Some("still_true".to_string())
        );
        assert_eq!(database.get_metadata("missing").unwrap(), None);
    }

    #[test]
    fn transcript_history_dao_writes_text_only_records() {
        let path = test_db_path("history");
        let database = Database::open(&path).unwrap();

        let id = database
            .insert_transcript_history(NewTranscriptHistory {
                raw_text: "raw words",
                final_text: "final words",
                output_style: "raw",
                recognition_started_at: "1700000000",
                recognition_duration_ms: 1200,
                injected: true,
                error_code: None,
                error_summary: None,
            })
            .unwrap();

        assert!(id > 0);
        assert_eq!(database.count_transcript_history().unwrap(), 1);
    }

    #[test]
    fn transcript_history_can_be_cleared() {
        let path = test_db_path("clear-history");
        let database = Database::open(&path).unwrap();
        database
            .insert_transcript_history(NewTranscriptHistory {
                raw_text: "raw words",
                final_text: "final words",
                output_style: "clean",
                recognition_started_at: "1700000000",
                recognition_duration_ms: 900,
                injected: false,
                error_code: Some("INJECT_FAILED"),
                error_summary: Some("target app rejected paste"),
            })
            .unwrap();

        let deleted = database.clear_transcript_history().unwrap();

        assert_eq!(deleted, 1);
        assert_eq!(database.count_transcript_history().unwrap(), 0);
    }

    #[test]
    fn transcript_history_item_can_be_deleted_by_id() {
        let path = test_db_path("delete-history-item");
        let database = Database::open(&path).unwrap();
        let first_id = database
            .insert_transcript_history(NewTranscriptHistory {
                raw_text: "first",
                final_text: "first",
                output_style: "raw",
                recognition_started_at: "1700000000",
                recognition_duration_ms: 300,
                injected: true,
                error_code: None,
                error_summary: None,
            })
            .unwrap();
        database
            .insert_transcript_history(NewTranscriptHistory {
                raw_text: "second",
                final_text: "second",
                output_style: "clean",
                recognition_started_at: "1700000001",
                recognition_duration_ms: 400,
                injected: false,
                error_code: None,
                error_summary: None,
            })
            .unwrap();

        let deleted = database.delete_transcript_history_item(first_id).unwrap();
        let page = database.get_transcript_history(10, 0).unwrap();

        assert_eq!(deleted, 1);
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].raw_text, "second");
    }

    #[test]
    fn transcript_history_listing_excludes_failed_records_from_results_and_total() {
        let path = test_db_path("history-page");
        let database = Database::open(&path).unwrap();

        let first_id = database
            .insert_transcript_history(NewTranscriptHistory {
                raw_text: "oldest",
                final_text: "oldest",
                output_style: "raw",
                recognition_started_at: "1700000000",
                recognition_duration_ms: 100,
                injected: true,
                error_code: None,
                error_summary: None,
            })
            .unwrap();
        let second_id = database
            .insert_transcript_history(NewTranscriptHistory {
                raw_text: "middle",
                final_text: "middle",
                output_style: "clean",
                recognition_started_at: "1700000001",
                recognition_duration_ms: 200,
                injected: false,
                error_code: Some("ASR_EMPTY"),
                error_summary: Some("no text"),
            })
            .unwrap();
        let third_id = database
            .insert_transcript_history(NewTranscriptHistory {
                raw_text: "latest",
                final_text: "latest",
                output_style: "formal",
                recognition_started_at: "1700000002",
                recognition_duration_ms: 300,
                injected: true,
                error_code: None,
                error_summary: None,
            })
            .unwrap();

        let connection = database.connection.lock().unwrap();
        connection
            .execute(
                "UPDATE transcript_history SET created_at = ?1 WHERE id = ?2",
                params![1_i64, first_id],
            )
            .unwrap();
        connection
            .execute(
                "UPDATE transcript_history SET created_at = ?1 WHERE id = ?2",
                params![2_i64, second_id],
            )
            .unwrap();
        connection
            .execute(
                "UPDATE transcript_history SET created_at = ?1 WHERE id = ?2",
                params![3_i64, third_id],
            )
            .unwrap();
        drop(connection);

        let page = database.get_transcript_history(2, 1).unwrap();

        assert_eq!(page.total, 2);
        assert_eq!(page.limit, 2);
        assert_eq!(page.offset, 1);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].raw_text, "oldest");
    }

    #[test]
    fn reapplying_failed_history_cleanup_migration_removes_legacy_failed_rows() {
        let path = test_db_path("cleanup-failed-history-migration");
        let database = Database::open(&path).unwrap();

        database
            .insert_transcript_history(NewTranscriptHistory {
                raw_text: "failed",
                final_text: "",
                output_style: "raw",
                recognition_started_at: "1700000000",
                recognition_duration_ms: 150,
                injected: false,
                error_code: Some("ASR_EMPTY"),
                error_summary: Some("No speech text was recognized."),
            })
            .unwrap();

        let connection = database.connection.lock().unwrap();
        connection
            .execute("DELETE FROM migration_history WHERE id = 3", [])
            .unwrap();
        drop(connection);

        database.apply_migrations().unwrap();

        assert_eq!(database.count_transcript_history().unwrap(), 0);
    }

    #[test]
    fn cleanup_transcript_history_removes_only_expired_rows() {
        let path = test_db_path("cleanup-history");
        let database = Database::open(&path).unwrap();

        let expired_id = database
            .insert_transcript_history(NewTranscriptHistory {
                raw_text: "expired",
                final_text: "expired",
                output_style: "raw",
                recognition_started_at: "1700000000",
                recognition_duration_ms: 100,
                injected: false,
                error_code: Some("ASR_FAILED"),
                error_summary: Some("offline"),
            })
            .unwrap();
        let fresh_id = database
            .insert_transcript_history(NewTranscriptHistory {
                raw_text: "fresh",
                final_text: "fresh",
                output_style: "clean",
                recognition_started_at: "1700000001",
                recognition_duration_ms: 150,
                injected: true,
                error_code: None,
                error_summary: None,
            })
            .unwrap();

        let cutoff = now_i64() - 2 * 24 * 60 * 60;
        let connection = database.connection.lock().unwrap();
        connection
            .execute(
                "UPDATE transcript_history SET created_at = ?1 WHERE id = ?2",
                params![cutoff, expired_id],
            )
            .unwrap();
        connection
            .execute(
                "UPDATE transcript_history SET created_at = ?1 WHERE id = ?2",
                params![now_i64(), fresh_id],
            )
            .unwrap();
        drop(connection);

        let deleted = database.cleanup_transcript_history(1).unwrap();
        let page = database.get_transcript_history(10, 0).unwrap();

        assert_eq!(deleted, 1);
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].raw_text, "fresh");
    }
}
