use crate::storage_error::StorageError;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct SqliteHandle {
    pub conn: Mutex<Connection>,
}

impl SqliteHandle {
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let handle = Self { conn: Mutex::new(conn) };
        handle.migrate()?;
        Ok(handle)
    }

    pub fn migrate(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS connections (id TEXT PRIMARY KEY, name TEXT NOT NULL, kind TEXT NOT NULL, endpoint TEXT NOT NULL, insecure INTEGER NOT NULL, credential_ref TEXT, created_at TEXT NOT NULL, last_connected_at TEXT);
            CREATE TABLE IF NOT EXISTS repo_cache (connection_id TEXT NOT NULL, query TEXT NOT NULL, payload_json TEXT NOT NULL, fetched_at TEXT NOT NULL, PRIMARY KEY (connection_id, query));
            CREATE TABLE IF NOT EXISTS tag_cache (connection_id TEXT NOT NULL, repository TEXT NOT NULL, payload_json TEXT NOT NULL, fetched_at TEXT NOT NULL, PRIMARY KEY (connection_id, repository));
            CREATE TABLE IF NOT EXISTS manifest_cache (digest TEXT PRIMARY KEY, payload_json TEXT NOT NULL, size_bytes INTEGER NOT NULL, fetched_at TEXT NOT NULL, last_accessed TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS pull_history (id TEXT PRIMARY KEY, connection_id TEXT NOT NULL, repo TEXT NOT NULL, tag TEXT NOT NULL, digest TEXT NOT NULL, output_path TEXT NOT NULL, size_bytes INTEGER NOT NULL, started_at TEXT NOT NULL, finished_at TEXT NOT NULL, status TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
        "#)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn migrate_creates_tables() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        let h = SqliteHandle::open(&path).unwrap();
        let conn = h.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('connections','repo_cache','tag_cache','manifest_cache','pull_history','settings')", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 6);
    }
}
