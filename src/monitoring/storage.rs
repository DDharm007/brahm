//! Persistent SQLite & Modular Event Storage Subsystem (Milestone 06.3).
//!
//! Provides the backend-agnostic `EventStore` trait and a concrete `SqliteEventStore` implementation
//! with versioned migrations and indexes. Never stores arbitrary repository source code contents.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::models::{EventSeverity, EventSource, EventType, IntegrityRecord, MonitoringEvent};
use crate::deception::decoy::DecoyArtifact;
use crate::deception::InspectionSession;
use crate::errors::{BrahmError, Result};

/// Backend-agnostic event persistence contract allowing future PostgreSQL migration.
pub trait EventStore: Send + Sync {
    fn record_event(&self, event: &MonitoringEvent) -> Result<()>;
    fn record_events_batch(&self, events: &[MonitoringEvent]) -> Result<()>;
    fn get_event(&self, event_id: &str) -> Result<Option<MonitoringEvent>>;
    fn list_events(
        &self,
        repository_id: Option<&str>,
        session_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<MonitoringEvent>>;
    fn record_session_ref(&self, session: &InspectionSession) -> Result<()>;
    fn record_decoy_ref(&self, repository_id: &str, decoy: &DecoyArtifact) -> Result<()>;
    fn record_integrity(&self, record: &IntegrityRecord) -> Result<()>;
    fn latest_integrity(&self, repository_id: &str) -> Result<Option<IntegrityRecord>>;
    fn count_events(&self) -> Result<usize>;
}

/// Thread-safe local SQLite event store backed by `rusqlite` bundled C library.
#[derive(Clone)]
pub struct SqliteEventStore {
    conn: Arc<Mutex<Connection>>,
}

impl std::fmt::Debug for SqliteEventStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteEventStore").finish()
    }
}

impl SqliteEventStore {
    /// Open or create a persistent SQLite database at `db_path` and apply migrations.
    pub fn open<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let p = db_path.as_ref();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(p).map_err(|e| {
            BrahmError::Database(format!("Failed opening SQLite DB '{}': {}", p.display(), e))
        })?;

        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        store.migrate()?;
        Ok(store)
    }

    /// Open an isolated in-memory SQLite database (ideal for testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| BrahmError::Database(format!("Failed opening in-memory SQLite: {}", e)))?;
        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        store.migrate()?;
        Ok(store)
    }

    /// Execute deterministic SQLite schema migrations.
    fn migrate(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| BrahmError::Database(format!("DB lock poisoned: {}", e)))?;

        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS monitoring_events (
                event_id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                repository_id TEXT NOT NULL,
                session_id TEXT,
                artifact_id TEXT,
                timestamp TEXT NOT NULL,
                source TEXT NOT NULL,
                severity TEXT NOT NULL,
                correlation_id TEXT NOT NULL,
                metadata_json TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_events_repo_id ON monitoring_events(repository_id);
            CREATE INDEX IF NOT EXISTS idx_events_session_id ON monitoring_events(session_id);
            CREATE INDEX IF NOT EXISTS idx_events_artifact_id ON monitoring_events(artifact_id);
            CREATE INDEX IF NOT EXISTS idx_events_timestamp ON monitoring_events(timestamp);
            CREATE INDEX IF NOT EXISTS idx_events_type ON monitoring_events(event_type);

            CREATE TABLE IF NOT EXISTS session_refs (
                session_id TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL,
                status TEXT NOT NULL,
                policy_id TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS decoy_refs (
                artifact_id TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL,
                relative_path TEXT NOT NULL,
                fingerprint TEXT NOT NULL,
                canary_token TEXT
            );

            CREATE TABLE IF NOT EXISTS integrity_records (
                record_id TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL,
                checked_at TEXT NOT NULL,
                baseline_matches INTEGER NOT NULL,
                payload_json TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_integrity_repo_time ON integrity_records(repository_id, checked_at);

            CREATE TABLE IF NOT EXISTS monitoring_config (
                config_key TEXT PRIMARY KEY,
                config_value TEXT NOT NULL
            );
            "#,
        )
        .map_err(|e| BrahmError::Database(format!("SQLite migration failed: {}", e)))?;

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM schema_migrations WHERE version = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|e| BrahmError::Database(format!("Migration check failed: {}", e)))?;

        if count == 0 {
            conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                params![1i64, Utc::now().to_rfc3339()],
            )
            .map_err(|e| BrahmError::Database(format!("Failed recording migration v1: {}", e)))?;
        }

        Ok(())
    }

    /// Set a key-value entry in `monitoring_config`.
    pub fn set_config(&self, key: &str, value: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| BrahmError::Database(format!("DB lock poisoned: {}", e)))?;
        conn.execute(
            "INSERT OR REPLACE INTO monitoring_config (config_key, config_value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| BrahmError::Database(format!("Failed writing monitoring_config: {}", e)))?;
        Ok(())
    }
}

impl EventStore for SqliteEventStore {
    fn record_event(&self, event: &MonitoringEvent) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| BrahmError::Database(format!("DB lock poisoned: {}", e)))?;

        let meta_json = serde_json::to_string(&event.metadata)?;
        conn.execute(
            r#"
            INSERT OR IGNORE INTO monitoring_events (
                event_id, event_type, repository_id, session_id, artifact_id,
                timestamp, source, severity, correlation_id, metadata_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                event.event_id,
                event.event_type.to_string(),
                event.repository_id,
                event.session_id,
                event.artifact_id,
                event.timestamp.to_rfc3339(),
                event.source.to_string(),
                event.severity.to_string(),
                event.correlation_id,
                meta_json
            ],
        )
        .map_err(|e| BrahmError::Database(format!("Failed inserting event: {}", e)))?;
        Ok(())
    }

    fn record_events_batch(&self, events: &[MonitoringEvent]) -> Result<()> {
        for evt in events {
            self.record_event(evt)?;
        }
        Ok(())
    }

    fn get_event(&self, event_id: &str) -> Result<Option<MonitoringEvent>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| BrahmError::Database(format!("DB lock poisoned: {}", e)))?;

        let mut stmt = conn
            .prepare(
                "SELECT event_id, event_type, repository_id, session_id, artifact_id, timestamp, source, severity, correlation_id, metadata_json FROM monitoring_events WHERE event_id = ?1",
            )
            .map_err(|e| BrahmError::Database(format!("Prepare failed: {}", e)))?;

        let mut rows = stmt
            .query_map(params![event_id], parse_event_row)
            .map_err(|e| BrahmError::Database(format!("Query failed: {}", e)))?;

        if let Some(row) = rows.next() {
            let evt = row.map_err(|e| BrahmError::Database(format!("Row decode failed: {}", e)))?;
            Ok(Some(evt))
        } else {
            Ok(None)
        }
    }

    fn list_events(
        &self,
        repository_id: Option<&str>,
        session_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<MonitoringEvent>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| BrahmError::Database(format!("DB lock poisoned: {}", e)))?;

        let limit_i64 = (limit.max(1)) as i64;
        let mut results = Vec::new();

        match (repository_id, session_id) {
            (Some(repo), Some(sess)) => {
                let mut stmt = conn
                    .prepare("SELECT event_id, event_type, repository_id, session_id, artifact_id, timestamp, source, severity, correlation_id, metadata_json FROM monitoring_events WHERE repository_id = ?1 AND session_id = ?2 ORDER BY timestamp ASC LIMIT ?3")
                    .map_err(|e| BrahmError::Database(format!("Prepare failed: {}", e)))?;
                let rows = stmt
                    .query_map(params![repo, sess, limit_i64], parse_event_row)
                    .map_err(|e| BrahmError::Database(format!("Query failed: {}", e)))?;
                for r in rows {
                    results.push(r.map_err(|e| BrahmError::Database(e.to_string()))?);
                }
            }
            (Some(repo), None) => {
                let mut stmt = conn
                    .prepare("SELECT event_id, event_type, repository_id, session_id, artifact_id, timestamp, source, severity, correlation_id, metadata_json FROM monitoring_events WHERE repository_id = ?1 ORDER BY timestamp ASC LIMIT ?2")
                    .map_err(|e| BrahmError::Database(format!("Prepare failed: {}", e)))?;
                let rows = stmt
                    .query_map(params![repo, limit_i64], parse_event_row)
                    .map_err(|e| BrahmError::Database(format!("Query failed: {}", e)))?;
                for r in rows {
                    results.push(r.map_err(|e| BrahmError::Database(e.to_string()))?);
                }
            }
            (None, Some(sess)) => {
                let mut stmt = conn
                    .prepare("SELECT event_id, event_type, repository_id, session_id, artifact_id, timestamp, source, severity, correlation_id, metadata_json FROM monitoring_events WHERE session_id = ?1 ORDER BY timestamp ASC LIMIT ?2")
                    .map_err(|e| BrahmError::Database(format!("Prepare failed: {}", e)))?;
                let rows = stmt
                    .query_map(params![sess, limit_i64], parse_event_row)
                    .map_err(|e| BrahmError::Database(format!("Query failed: {}", e)))?;
                for r in rows {
                    results.push(r.map_err(|e| BrahmError::Database(e.to_string()))?);
                }
            }
            (None, None) => {
                let mut stmt = conn
                    .prepare("SELECT event_id, event_type, repository_id, session_id, artifact_id, timestamp, source, severity, correlation_id, metadata_json FROM monitoring_events ORDER BY timestamp ASC LIMIT ?1")
                    .map_err(|e| BrahmError::Database(format!("Prepare failed: {}", e)))?;
                let rows = stmt
                    .query_map(params![limit_i64], parse_event_row)
                    .map_err(|e| BrahmError::Database(format!("Query failed: {}", e)))?;
                for r in rows {
                    results.push(r.map_err(|e| BrahmError::Database(e.to_string()))?);
                }
            }
        }

        Ok(results)
    }

    fn record_session_ref(&self, session: &InspectionSession) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| BrahmError::Database(format!("DB lock poisoned: {}", e)))?;

        conn.execute(
            "INSERT OR REPLACE INTO session_refs (session_id, repository_id, status, policy_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                session.session_id,
                session.repository_id,
                format!("{:?}", session.status),
                session.policy_id,
                session.created_at.to_rfc3339()
            ],
        )
        .map_err(|e| BrahmError::Database(format!("Failed recording session ref: {}", e)))?;
        Ok(())
    }

    fn record_decoy_ref(&self, repository_id: &str, decoy: &DecoyArtifact) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| BrahmError::Database(format!("DB lock poisoned: {}", e)))?;

        conn.execute(
            "INSERT OR REPLACE INTO decoy_refs (artifact_id, repository_id, relative_path, fingerprint, canary_token) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                decoy.artifact_id,
                repository_id,
                decoy.synthetic_relative_path,
                decoy.integrity_fingerprint,
                decoy.canary_token
            ],
        )
        .map_err(|e| BrahmError::Database(format!("Failed recording decoy ref: {}", e)))?;
        Ok(())
    }

    fn record_integrity(&self, record: &IntegrityRecord) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| BrahmError::Database(format!("DB lock poisoned: {}", e)))?;

        let payload_json = serde_json::to_string(record)?;
        conn.execute(
            "INSERT OR REPLACE INTO integrity_records (record_id, repository_id, checked_at, baseline_matches, payload_json) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                record.record_id,
                record.repository_id,
                record.checked_at.to_rfc3339(),
                if record.baseline_matches { 1i64 } else { 0i64 },
                payload_json
            ],
        )
        .map_err(|e| BrahmError::Database(format!("Failed recording integrity: {}", e)))?;
        Ok(())
    }

    fn latest_integrity(&self, repository_id: &str) -> Result<Option<IntegrityRecord>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| BrahmError::Database(format!("DB lock poisoned: {}", e)))?;

        let mut stmt = conn
            .prepare("SELECT payload_json FROM integrity_records WHERE repository_id = ?1 ORDER BY checked_at DESC LIMIT 1")
            .map_err(|e| BrahmError::Database(format!("Prepare failed: {}", e)))?;

        let mut rows = stmt
            .query_map(params![repository_id], |row| row.get::<_, String>(0))
            .map_err(|e| BrahmError::Database(format!("Query failed: {}", e)))?;

        if let Some(row) = rows.next() {
            let json_str = row.map_err(|e| BrahmError::Database(e.to_string()))?;
            let rec: IntegrityRecord = serde_json::from_str(&json_str)?;
            Ok(Some(rec))
        } else {
            Ok(None)
        }
    }

    fn count_events(&self) -> Result<usize> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| BrahmError::Database(format!("DB lock poisoned: {}", e)))?;
        let cnt: i64 = conn
            .query_row("SELECT COUNT(*) FROM monitoring_events", [], |r| r.get(0))
            .map_err(|e| BrahmError::Database(e.to_string()))?;
        Ok(cnt as usize)
    }
}

fn parse_event_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MonitoringEvent> {
    let event_id: String = row.get(0)?;
    let event_type_str: String = row.get(1)?;
    let repository_id: String = row.get(2)?;
    let session_id: Option<String> = row.get(3)?;
    let artifact_id: Option<String> = row.get(4)?;
    let ts_str: String = row.get(5)?;
    let source_str: String = row.get(6)?;
    let sev_str: String = row.get(7)?;
    let correlation_id: String = row.get(8)?;
    let meta_str: String = row.get(9)?;

    let event_type = EventType::from_db_str(&event_type_str).unwrap_or(EventType::DecoyAccessed);
    let timestamp = DateTime::parse_from_rfc3339(&ts_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    let source = EventSource::from_db_str(&source_str);
    let severity = EventSeverity::from_db_str(&sev_str);
    let metadata: BTreeMap<String, String> = serde_json::from_str(&meta_str).unwrap_or_default();

    Ok(MonitoringEvent {
        event_id,
        event_type,
        repository_id,
        session_id,
        artifact_id,
        timestamp,
        source,
        severity,
        correlation_id,
        metadata,
    })
}
