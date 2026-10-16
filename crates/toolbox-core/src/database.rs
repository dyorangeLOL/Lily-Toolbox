use crate::{modules::Manifest, AppError};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{path::Path, time::Duration};

pub struct Migration {
    pub version: u32,
    pub sql: &'static str,
}

fn checksum(sql: &str) -> String {
    // Portable databases must verify identically after LF/CRLF checkouts.
    format!("{:x}", Sha256::digest(sql.replace("\r\n", "\n").as_bytes()))
}
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        sql: include_str!("../migrations/001_core.sql"),
    },
    Migration {
        version: 2,
        sql: include_str!("../migrations/002_grants.sql"),
    },
];

pub fn storage_error(e: rusqlite::Error) -> AppError {
    match e.sqlite_error_code() {
        Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked) => {
            AppError::new("DATABASE_BUSY")
        }
        _ => AppError::new("STORAGE_FAILED"),
    }
}

pub struct Database {
    pub(crate) connection: Connection,
}

impl Database {
    pub fn foreign_keys_enabled(&self) -> Result<bool, AppError> {
        self.connection
            .query_row("PRAGMA foreign_keys", [], |r| r.get::<_, i32>(0))
            .map(|v| v == 1)
            .map_err(storage_error)
    }
    pub fn open(path: &Path) -> Result<Self, AppError> {
        let mut connection = Connection::open(path).map_err(storage_error)?;
        connection
            .busy_timeout(Duration::from_secs(3))
            .map_err(storage_error)?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(storage_error)?;
        let mode: String = connection
            .query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))
            .map_err(storage_error)?;
        if mode != "wal" {
            return Err(AppError::new("STORAGE_FAILED"));
        }
        connection
            .pragma_update(None, "trusted_schema", "OFF")
            .map_err(storage_error)?;
        migrate(&mut connection, MIGRATIONS)?;
        let mut result = Self { connection };
        result.sync_builtins()?;
        Ok(result)
    }

    fn sync_builtins(&mut self) -> Result<(), AppError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        for m in crate::modules::builtins() {
            m.validate()?;
            let json = serde_json::to_string(&m).map_err(|_| AppError::new("INVALID_DATA"))?;
            tx.execute("INSERT INTO modules(id,version,kind,enabled,installed_path,manifest_version,manifest_json) VALUES(?1,?2,'builtin',1,NULL,1,?3) ON CONFLICT(id) DO UPDATE SET version=excluded.version,manifest_json=excluded.manifest_json", params![m.id,m.version,json]).map_err(storage_error)?;
        }
        tx.commit().map_err(storage_error)
    }

    pub fn modules(&self) -> Result<Vec<Manifest>, AppError> {
        let mut stmt = self
            .connection
            .prepare(
                "SELECT manifest_json FROM modules WHERE kind='builtin' AND enabled=1 ORDER BY id",
            )
            .map_err(storage_error)?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(storage_error)?;
        let trusted = crate::modules::builtins();
        rows.map(|row| {
            let m: Manifest = serde_json::from_str(&row.map_err(storage_error)?)
                .map_err(|_| AppError::new("INVALID_DATA"))?;
            m.validate()?;
            // Database content cannot invent an executable entry or new permissions.
            if !trusted.contains(&m) {
                return Err(AppError::new("INVALID_DATA"));
            }
            Ok(m)
        })
        .collect()
    }

    pub fn set_setting(&self, key: &str, value: &Value) -> Result<(), AppError> {
        if !crate::contracts::valid_id(key) || value.to_string().len() > 65536 {
            return Err(AppError::new("INVALID_DATA"));
        }
        self.connection.execute("INSERT INTO settings(key,value_json,scope,updated_at) VALUES(?1,?2,'global',unixepoch()) ON CONFLICT(key,scope) DO UPDATE SET value_json=excluded.value_json,updated_at=excluded.updated_at",params![key,value.to_string()]).map_err(storage_error)?;
        Ok(())
    }
    pub fn setting(&self, key: &str) -> Result<Option<Value>, AppError> {
        let json: Option<String> = self
            .connection
            .query_row(
                "SELECT value_json FROM settings WHERE key=?1 AND scope='global'",
                [key],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        json.map(|j| serde_json::from_str(&j).map_err(|_| AppError::new("INVALID_DATA")))
            .transpose()
    }

    // Only fixed metadata enters logs; request params / raw errors / paths never do.
    pub(crate) fn audit(
        &self,
        subject: &str,
        request_id: &str,
        command: &str,
        outcome: &str,
    ) -> Result<(), AppError> {
        let details = serde_json::json!({"request_id":request_id,"command":command});
        self.connection.execute("INSERT INTO audit_events(event_type,subject_id,outcome,timestamp,safe_details_json) VALUES('command',?1,?2,unixepoch(),?3)", params![subject,outcome,details.to_string()]).map_err(storage_error)?;
        Ok(())
    }
}

pub fn migrate(connection: &mut Connection, migrations: &[Migration]) -> Result<(), AppError> {
    if migrations
        .iter()
        .enumerate()
        .any(|(i, m)| m.version != i as u32 + 1)
    {
        return Err(AppError::new("SCHEMA_MISMATCH"));
    }
    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage_error)?;
    tx.execute_batch("CREATE TABLE IF NOT EXISTS schema_migrations(version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL, checksum TEXT NOT NULL)").map_err(|_| AppError::new("MIGRATION_FAILED"))?;
    let applied: Vec<(u32, String)> = {
        let mut statement = tx
            .prepare("SELECT version,checksum FROM schema_migrations ORDER BY version")
            .map_err(|_| AppError::new("SCHEMA_MISMATCH"))?;
        let rows = statement
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|_| AppError::new("SCHEMA_MISMATCH"))?;
        rows.collect::<rusqlite::Result<_>>()
            .map_err(|_| AppError::new("SCHEMA_MISMATCH"))?
    };
    for (i, (version, recorded_checksum)) in applied.iter().enumerate() {
        let m = migrations
            .get(i)
            .filter(|m| m.version == *version)
            .ok_or_else(|| AppError::new("SCHEMA_MISMATCH"))?;
        if checksum(m.sql) != *recorded_checksum {
            return Err(AppError::new("SCHEMA_MISMATCH"));
        }
    }
    for m in &migrations[applied.len()..] {
        tx.execute_batch(m.sql)
            .map_err(|_| AppError::new("MIGRATION_FAILED"))?;
        tx.execute(
            "INSERT INTO schema_migrations(version,applied_at,checksum) VALUES(?1,unixepoch(),?2)",
            params![m.version, checksum(m.sql)],
        )
        .map_err(|_| AppError::new("MIGRATION_FAILED"))?;
    }
    tx.commit().map_err(|_| AppError::new("MIGRATION_FAILED"))
}
