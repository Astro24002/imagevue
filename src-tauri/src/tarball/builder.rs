use crate::pull_error::PullError;
use serde_json::json;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

pub struct TarballBuilder {
    #[allow(dead_code)]
    path: PathBuf,
    file: BufWriter<File>,
    config_name: Option<String>,
    repos: Vec<String>,
    layers: Vec<String>,
}

impl TarballBuilder {
    pub fn new(path: &Path) -> Result<Self, PullError> {
        let f = File::create(path).map_err(|e| PullError::WriteTar(e.to_string()))?;
        Ok(Self { path: path.to_path_buf(), file: BufWriter::new(f), config_name: None, repos: vec![], layers: vec![] })
    }

    pub fn add_config(&mut self, digest: &str, bytes: &[u8]) -> Result<(), PullError> {
        let name = strip_prefix(digest);
        self.write_entry(&format!("{name}.json"), bytes)?;
        self.config_name = Some(name);
        Ok(())
    }

    pub fn add_layer(&mut self, digest: &str, bytes: &[u8]) -> Result<(), PullError> {
        let entry_path = format!("{digest}/layer.tar");
        self.write_entry(&entry_path, bytes)?;
        self.layers.push(entry_path);
        Ok(())
    }

    pub fn set_repos(&mut self, repos: Vec<String>) { self.repos = repos; }

    pub fn finish(mut self) -> Result<(), PullError> {
        let cfg = self.config_name.as_ref().ok_or_else(|| PullError::WriteTar("no config".into()))?;
        let manifest = json!([{ "Config": format!("{cfg}.json"), "Repos": self.repos.clone(), "Layers": self.layers.clone() }]);
        let mb = serde_json::to_vec_pretty(&manifest).map_err(|e| PullError::WriteTar(e.to_string()))?;
        self.write_entry("manifest.json", &mb)?;
        let mut repos_map = serde_json::Map::new();
        let mut tags_map = serde_json::Map::new();
        for r in &self.repos {
            let (repo, tag) = r.split_once(':').unwrap_or((r.as_str(), ""));
            let entry = tags_map.entry(repo.to_string()).or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
            if let serde_json::Value::Object(m) = entry { m.insert(tag.to_string(), json!({})); }
        }
        repos_map.insert("_".into(), serde_json::Value::Object(tags_map));
        let rb = serde_json::to_vec_pretty(&repos_map).map_err(|e| PullError::WriteTar(e.to_string()))?;
        self.write_entry("repositories", &rb)?;
        self.file.flush().map_err(|e| PullError::WriteTar(e.to_string()))?;
        self.file.into_inner().map_err(|e| PullError::WriteTar(e.to_string()))?.sync_all().map_err(|e| PullError::WriteTar(e.to_string()))?;
        Ok(())
    }

    fn write_entry(&mut self, name: &str, bytes: &[u8]) -> Result<(), PullError> {
        let mut header = tar::Header::new_gnu();
        header.set_path(name).map_err(|e| PullError::WriteTar(e.to_string()))?;
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        self.file.write_all(header.as_bytes()).map_err(|e| PullError::WriteTar(e.to_string()))?;
        self.file.write_all(bytes).map_err(|e| PullError::WriteTar(e.to_string()))?;
        let pad = (512 - (bytes.len() % 512)) % 512;
        let zeros = vec![0u8; pad];
        self.file.write_all(&zeros).map_err(|e| PullError::WriteTar(e.to_string()))?;
        Ok(())
    }
}

fn strip_prefix(digest: &str) -> String { digest.strip_prefix("sha256:").unwrap_or(digest).to_string() }

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn builds_docker_tar() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("out.tar");
        let mut b = TarballBuilder::new(&path).unwrap();
        b.add_config("sha256:configdigest", b"{}").unwrap();
        b.add_layer("sha256:layer1", b"LAYER1").unwrap();
        b.add_layer("sha256:layer2", b"LAYER2").unwrap();
        b.set_repos(vec!["lib/nginx:v1".into()]);
        b.finish().unwrap();
        let f = std::fs::File::open(&path).unwrap();
        let mut a = tar::Archive::new(f);
        let names: Vec<String> = a.entries().unwrap().map(|e| e.unwrap().path().unwrap().to_str().unwrap().to_string()).collect();
        assert!(names.contains(&"manifest.json".to_string()));
        assert!(names.contains(&"configdigest.json".to_string()));
        assert!(names.contains(&"sha256:layer1/layer.tar".to_string()));
        assert!(names.contains(&"sha256:layer2/layer.tar".to_string()));
        assert!(names.contains(&"repositories".to_string()));
    }
}
