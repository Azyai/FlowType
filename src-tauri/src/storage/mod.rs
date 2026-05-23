use crate::error::AppResult;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

const MIGRATIONS: &[Migration] = &[Migration {
    id: 1,
    name: "create_app_metadata",
    sql: r#"
        CREATE TABLE IF NOT EXISTS app_metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
    "#,
}];

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

#[derive(Debug)]
pub struct Database {
    path: PathBuf,
    connection: Mutex<Connection>,
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
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
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
        assert_eq!(health.applied_migrations, 1);
    }

    #[test]
    fn migrations_are_idempotent() {
        let path = test_db_path("idempotent");
        let database = Database::open(&path).unwrap();

        database.apply_migrations().unwrap();
        database.apply_migrations().unwrap();

        assert_eq!(database.count_migrations().unwrap(), 1);
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
}
