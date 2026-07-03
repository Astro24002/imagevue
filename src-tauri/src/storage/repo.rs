use crate::oci::auth::RegistryKind;
use crate::oci::types::{ManifestSummary, Repository, Tag};
use crate::storage::sqlite::SqliteHandle;
use crate::storage_error::StorageError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryConnection {
    pub id: Uuid,
    pub name: String,
    pub kind: RegistryKind,
    pub endpoint: String,
    pub insecure: bool,
    pub credential_ref: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_connected_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewConnection {
    pub name: String,
    pub kind: RegistryKind,
    pub endpoint: String,
    pub insecure: bool,
    pub credential_ref: Option<String>,
}

pub struct ConnectionRepo<'a> { handle: &'a SqliteHandle }

impl<'a> ConnectionRepo<'a> {
    pub fn new(handle: &'a SqliteHandle) -> Self { Self { handle } }

    pub fn create(&self, c: NewConnection) -> Result<RegistryConnection, StorageError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let conn = self.handle.conn.lock().unwrap();
        conn.execute("INSERT INTO connections (id, name, kind, endpoint, insecure, credential_ref, created_at, last_connected_at) VALUES (?,?,?,?,?,?,?,?)",
            rusqlite::params![id.to_string(), c.name, c.kind.as_str(), c.endpoint, c.insecure as i32, c.credential_ref, now.to_rfc3339(), None::<String>])?;
        Ok(RegistryConnection { id, name: c.name, kind: c.kind, endpoint: c.endpoint, insecure: c.insecure, credential_ref: c.credential_ref, created_at: now, last_connected_at: None })
    }

    pub fn update(&self, c: &RegistryConnection) -> Result<(), StorageError> {
        let conn = self.handle.conn.lock().unwrap();
        conn.execute("UPDATE connections SET name=?, kind=?, endpoint=?, insecure=?, credential_ref=?, last_connected_at=? WHERE id=?",
            rusqlite::params![c.name, c.kind.as_str(), c.endpoint, c.insecure as i32, c.credential_ref, c.last_connected_at.map(|d| d.to_rfc3339()), c.id.to_string()])?;
        Ok(())
    }

    pub fn delete(&self, id: Uuid) -> Result<(), StorageError> {
        let conn = self.handle.conn.lock().unwrap();
        conn.execute("DELETE FROM connections WHERE id=?", [id.to_string()])?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<RegistryConnection>, StorageError> {
        let conn = self.handle.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, kind, endpoint, insecure, credential_ref, created_at, last_connected_at FROM connections ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |r| Ok(row_to_connection(r)?))?;
        let mut out = Vec::new();
        for r in rows { out.push(r.map_err(|e| StorageError::Migration(e.to_string()))?); }
        Ok(out)
    }

    pub fn get(&self, id: Uuid) -> Result<RegistryConnection, StorageError> {
        let conn = self.handle.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, kind, endpoint, insecure, credential_ref, created_at, last_connected_at FROM connections WHERE id=?")?;
        let mut rows = stmt.query([id.to_string()])?;
        let Some(r) = rows.next().map_err(|e| StorageError::Migration(e.to_string()))? else { return Err(StorageError::NotFound(id.to_string())); };
        Ok(row_to_connection(r).map_err(|e| StorageError::Migration(e.to_string()))?)
    }

    pub fn mark_connected(&self, id: Uuid) -> Result<(), StorageError> {
        let conn = self.handle.conn.lock().unwrap();
        conn.execute("UPDATE connections SET last_connected_at=? WHERE id=?", rusqlite::params![Utc::now().to_rfc3339(), id.to_string()])?;
        Ok(())
    }
}

fn row_to_connection(r: &rusqlite::Row) -> Result<RegistryConnection, rusqlite::Error> {
    let id_s: String = r.get(0)?;
    let kind_s: String = r.get(2)?;
    let created_s: String = r.get(6)?;
    let last_s: Option<String> = r.get(7)?;
    let parse_dt = |s: &str| DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&Utc)).map_err(|e| rusqlite::Error::InvalidColumnType(0, e.to_string(), rusqlite::types::Type::Text));
    Ok(RegistryConnection {
        id: Uuid::parse_str(&id_s).map_err(|e| rusqlite::Error::InvalidColumnType(0, e.to_string(), rusqlite::types::Type::Text))?,
        name: r.get(1)?,
        kind: parse_kind(&kind_s)?,
        endpoint: r.get(3)?,
        insecure: r.get::<_, i32>(4)? != 0,
        credential_ref: r.get(5)?,
        created_at: parse_dt(&created_s)?,
        last_connected_at: last_s.map(|s| parse_dt(&s)).transpose()?,
    })
}

fn parse_kind(s: &str) -> Result<RegistryKind, rusqlite::Error> {
    Ok(match s {
        "dockerHub" => RegistryKind::DockerHub,
        "ghcr" => RegistryKind::Ghcr,
        "quay" => RegistryKind::Quay,
        "gcr" => RegistryKind::Gcr,
        "generic" => RegistryKind::Generic,
        other => return Err(rusqlite::Error::InvalidColumnType(0, format!("unknown kind {other}"), rusqlite::types::Type::Text)),
    })
}

pub struct CacheRepo<'a> { handle: &'a SqliteHandle }

impl<'a> CacheRepo<'a> {
    pub fn new(handle: &'a SqliteHandle) -> Self { Self { handle } }

    pub fn put_repo_list(&self, connection_id: Uuid, query: &str, repos: &[Repository]) -> Result<(), StorageError> {
        let conn = self.handle.conn.lock().unwrap();
        let payload = serde_json::to_string(repos).map_err(|e| StorageError::Migration(e.to_string()))?;
        conn.execute("INSERT OR REPLACE INTO repo_cache (connection_id, query, payload_json, fetched_at) VALUES (?,?,?,?)",
            rusqlite::params![connection_id.to_string(), query, payload, Utc::now().to_rfc3339()])?;
        Ok(())
    }

    pub fn get_repo_list(&self, connection_id: Uuid, query: &str, max_age: Duration) -> Result<Option<Vec<Repository>>, StorageError> {
        let conn = self.handle.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT payload_json, fetched_at FROM repo_cache WHERE connection_id=? AND query=?")?;
        let mut rows = stmt.query(rusqlite::params![connection_id.to_string(), query])?;
        let Some(r) = rows.next().map_err(|e| StorageError::Migration(e.to_string()))? else { return Ok(None); };
        let payload: String = r.get(0).map_err(|e| StorageError::Migration(e.to_string()))?;
        let fetched_s: String = r.get(1).map_err(|e| StorageError::Migration(e.to_string()))?;
        let fetched = DateTime::parse_from_rfc3339(&fetched_s).map_err(|e| StorageError::Migration(e.to_string()))?.with_timezone(&Utc);
        if Utc::now().signed_duration_since(fetched) > chrono::Duration::from_std(max_age).unwrap_or(chrono::Duration::seconds(300)) {
            return Ok(None);
        }
        let list: Vec<Repository> = serde_json::from_str(&payload).map_err(|e| StorageError::Migration(e.to_string()))?;
        Ok(Some(list))
    }

    pub fn put_tag_list(&self, connection_id: Uuid, repo: &str, tags: &[Tag]) -> Result<(), StorageError> {
        let conn = self.handle.conn.lock().unwrap();
        let payload = serde_json::to_string(tags).map_err(|e| StorageError::Migration(e.to_string()))?;
        conn.execute("INSERT OR REPLACE INTO tag_cache (connection_id, repository, payload_json, fetched_at) VALUES (?,?,?,?)",
            rusqlite::params![connection_id.to_string(), repo, payload, Utc::now().to_rfc3339()])?;
        Ok(())
    }

    pub fn get_tag_list(&self, connection_id: Uuid, repo: &str, max_age: Duration) -> Result<Option<Vec<Tag>>, StorageError> {
        let conn = self.handle.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT payload_json, fetched_at FROM tag_cache WHERE connection_id=? AND repository=?")?;
        let mut rows = stmt.query(rusqlite::params![connection_id.to_string(), repo])?;
        let Some(r) = rows.next().map_err(|e| StorageError::Migration(e.to_string()))? else { return Ok(None); };
        let payload: String = r.get(0).map_err(|e| StorageError::Migration(e.to_string()))?;
        let fetched_s: String = r.get(1).map_err(|e| StorageError::Migration(e.to_string()))?;
        let fetched = DateTime::parse_from_rfc3339(&fetched_s).map_err(|e| StorageError::Migration(e.to_string()))?.with_timezone(&Utc);
        if Utc::now().signed_duration_since(fetched) > chrono::Duration::from_std(max_age).unwrap_or(chrono::Duration::seconds(300)) {
            return Ok(None);
        }
        let list: Vec<Tag> = serde_json::from_str(&payload).map_err(|e| StorageError::Migration(e.to_string()))?;
        Ok(Some(list))
    }

    pub fn put_manifest(&self, digest: &str, m: &ManifestSummary) -> Result<(), StorageError> {
        let conn = self.handle.conn.lock().unwrap();
        let payload = serde_json::to_string(m).map_err(|e| StorageError::Migration(e.to_string()))?;
        let size = payload.len() as i64;
        conn.execute("INSERT OR REPLACE INTO manifest_cache (digest, payload_json, size_bytes, fetched_at, last_accessed) VALUES (?,?,?,?,?)",
            rusqlite::params![digest, payload, size, Utc::now().to_rfc3339(), Utc::now().to_rfc3339()])?;
        Ok(())
    }

    pub fn get_manifest(&self, digest: &str) -> Result<Option<ManifestSummary>, StorageError> {
        let conn = self.handle.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT payload_json FROM manifest_cache WHERE digest=?")?;
        let mut rows = stmt.query([digest])?;
        let Some(r) = rows.next().map_err(|e| StorageError::Migration(e.to_string()))? else { return Ok(None); };
        let payload: String = r.get(0).map_err(|e| StorageError::Migration(e.to_string()))?;
        conn.execute("UPDATE manifest_cache SET last_accessed=? WHERE digest=?", [Utc::now().to_rfc3339(), digest.to_string()]).ok();
        let m: ManifestSummary = serde_json::from_str(&payload).map_err(|e| StorageError::Migration(e.to_string()))?;
        Ok(Some(m))
    }

    pub fn evict_manifests_above(&self, max_bytes: u64) -> Result<(), StorageError> {
        let conn = self.handle.conn.lock().unwrap();
        let total: i64 = conn.query_row("SELECT COALESCE(SUM(size_bytes),0) FROM manifest_cache", [], |r| r.get(0))?;
        if (total as u64) <= max_bytes { return Ok(()); }
        let digests: Vec<String> = conn.prepare("SELECT digest FROM manifest_cache ORDER BY last_accessed ASC")?
            .query_map([], |r| r.get::<_, String>(0))?
            .filter_map(|r| r.ok()).collect();
        let need = (total as u64) - max_bytes;
        let mut freed = 0u64;
        for d in digests {
            if freed >= need { break; }
            let size: i64 = conn.query_row("SELECT size_bytes FROM manifest_cache WHERE digest=?", [&d], |r| r.get(0)).unwrap_or(0);
            conn.execute("DELETE FROM manifest_cache WHERE digest=?", [&d])?;
            freed += size as u64;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oci::types::ArtifactKind;
    use tempfile::tempdir;

    #[test]
    fn connection_crud() {
        let dir = tempdir().unwrap();
        let h = SqliteHandle::open(&dir.path().join("t.db")).unwrap();
        let repo = ConnectionRepo::new(&h);
        let c = repo.create(NewConnection { name: "Docker Hub".into(), kind: RegistryKind::DockerHub, endpoint: "https://registry-1.docker.io".into(), insecure: false, credential_ref: None }).unwrap();
        assert_eq!(repo.list().unwrap().len(), 1);
        assert_eq!(repo.get(c.id).unwrap().name, "Docker Hub");
        repo.delete(c.id).unwrap();
        assert_eq!(repo.list().unwrap().len(), 0);
    }

    #[test]
    fn cache_repo_list_round_trip() {
        let dir = tempdir().unwrap();
        let h = SqliteHandle::open(&dir.path().join("t.db")).unwrap();
        let cache = CacheRepo::new(&h);
        let cid = Uuid::new_v4();
        cache.put_repo_list(cid, "", &[Repository { name: "a".into() }, Repository { name: "b".into() }]).unwrap();
        let got = cache.get_repo_list(cid, "", Duration::from_secs(300)).unwrap().unwrap();
        assert_eq!(got.len(), 2);
    }

    #[test]
    fn cache_tag_list_round_trip() {
        let dir = tempdir().unwrap();
        let h = SqliteHandle::open(&dir.path().join("t.db")).unwrap();
        let cache = CacheRepo::new(&h);
        let cid = Uuid::new_v4();
        cache.put_tag_list(cid, "lib/nginx", &[Tag { name: "v1".into(), digest: "sha256:a".into(), size: 1, updated_at: None, os: None, architecture: None, artifact_kind: ArtifactKind::Image }]).unwrap();
        let got = cache.get_tag_list(cid, "lib/nginx", Duration::from_secs(300)).unwrap().unwrap();
        assert_eq!(got.len(), 1);
    }
}
