use crate::oci::types::*;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported media type: {0}")]
    UnsupportedMediaType(String),
    #[error("missing field: {0}")]
    MissingField(&'static str),
    #[error("invalid schema version: {0}")]
    InvalidSchemaVersion(u32),
}

pub fn parse_manifest(raw: &[u8], content_type: &str) -> Result<ManifestSummary, ManifestError> {
    let kind = match content_type {
        "application/vnd.docker.distribution.manifest.v2+json" => ManifestKind::DockerV2,
        "application/vnd.docker.distribution.manifest.v1+json" => ManifestKind::DockerV1,
        "application/vnd.docker.distribution.manifest.list.v2+json" => ManifestKind::OciIndex,
        "application/vnd.oci.image.manifest.v1+json" => ManifestKind::OciManifest,
        "application/vnd.oci.image.index.v1+json" => ManifestKind::OciIndex,
        other => return Err(ManifestError::UnsupportedMediaType(other.to_string())),
    };

    #[derive(serde::Deserialize)]
    struct RawManifest {
        #[serde(default, rename = "schemaVersion")] schema_version: u32,
        #[serde(default)] config: Option<RawDescriptor>,
        #[serde(default)] layers: Vec<RawDescriptor>,
        #[serde(default)] manifests: Vec<RawManifestEntry>,
    }
    #[derive(serde::Deserialize)]
    struct RawDescriptor { #[serde(rename = "mediaType")] media_type: String, digest: String, size: u64 }
    #[derive(serde::Deserialize)]
    struct RawManifestEntry {
        #[serde(rename = "mediaType")] media_type: String, digest: String, size: u64,
        #[serde(default)] platform: Option<RawPlatform>,
    }
    #[derive(serde::Deserialize)]
    struct RawPlatform { architecture: String, os: String, #[serde(default)] variant: Option<String>, #[serde(default, rename = "os.version")] os_version: Option<String>, #[serde(default, rename = "os.features")] os_features: Option<Vec<String>> }

    let raw_obj: RawManifest = serde_json::from_slice(raw)?;
    let mut config_desc = None;
    let mut layer_descs = Vec::new();
    let mut platforms = Vec::new();
    let mut total_size = 0u64;

    match kind {
        ManifestKind::DockerV1 => {
            if raw_obj.schema_version != 1 { return Err(ManifestError::InvalidSchemaVersion(raw_obj.schema_version)); }
            let config = raw_obj.config.ok_or(ManifestError::MissingField("config"))?;
            config_desc = Some(Descriptor { media_type: config.media_type, digest: config.digest, size: config.size, urls: None, annotations: None });
            for l in raw_obj.layers { total_size += l.size; layer_descs.push(Descriptor { media_type: l.media_type, digest: l.digest, size: l.size, urls: None, annotations: None }); }
        }
        ManifestKind::DockerV2 | ManifestKind::OciManifest => {
            if raw_obj.schema_version != 2 { return Err(ManifestError::InvalidSchemaVersion(raw_obj.schema_version)); }
            let config = raw_obj.config.ok_or(ManifestError::MissingField("config"))?;
            config_desc = Some(Descriptor { media_type: config.media_type, digest: config.digest, size: config.size, urls: None, annotations: None });
            for l in raw_obj.layers { total_size += l.size; layer_descs.push(Descriptor { media_type: l.media_type, digest: l.digest, size: l.size, urls: None, annotations: None }); }
        }
        ManifestKind::OciIndex => {
            for m in raw_obj.manifests {
                if let Some(p) = m.platform { platforms.push(Platform { architecture: p.architecture, os: p.os, variant: p.variant, os_version: p.os_version, os_features: p.os_features }); }
                total_size += m.size;
            }
        }
        ManifestKind::Unknown => return Err(ManifestError::UnsupportedMediaType(content_type.into())),
    }

    let layer_count = layer_descs.len() as u32;
    let actual_digest = format!("sha256:{}", hex::encode(Sha256::digest(raw)));

    Ok(ManifestSummary {
        digest: actual_digest,
        media_type: content_type.to_string(),
        schema_version: raw_obj.schema_version,
        kind,
        total_size,
        layer_count,
        config_descriptor: config_desc,
        layer_descriptors: layer_descs,
        platforms,
        raw_json: String::from_utf8_lossy(raw).to_string(),
        artifact_kind: ArtifactKind::Image,
    })
}

pub fn parse_image_config(raw: &[u8]) -> Result<ImageConfig, ManifestError> {
    #[derive(serde::Deserialize)]
    struct Raw {
        architecture: String, os: String,
        #[serde(default)] created: Option<chrono::DateTime<chrono::Utc>>,
        #[serde(default)] author: Option<String>,
        #[serde(default)] config: RawConfig,
        #[serde(default)] history: Vec<RawHistory>,
        rootfs: RawRootFs,
    }
    #[derive(serde::Deserialize, Default)]
    struct RawConfig {
        #[serde(default, rename = "Env")] env: Vec<String>,
        #[serde(default)] cmd: Option<Vec<String>>,
        #[serde(default)] entrypoint: Option<Vec<String>>,
        #[serde(default)] working_dir: Option<String>,
        #[serde(default)] exposed_ports: std::collections::HashMap<String, serde_json::Value>,
        #[serde(default)] labels: std::collections::HashMap<String, String>,
    }
    #[derive(serde::Deserialize)]
    struct RawHistory {
        #[serde(default)] created: Option<chrono::DateTime<chrono::Utc>>,
        #[serde(default)] author: Option<String>,
        #[serde(default)] created_by: Option<String>,
        #[serde(default)] empty_layer: Option<bool>,
        #[serde(default)] comment: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct RawRootFs { #[serde(rename = "type")] kind: String, diff_ids: Vec<String> }

    let r: Raw = serde_json::from_slice(raw)?;
    let digest = format!("sha256:{}", hex::encode(Sha256::digest(raw)));

    Ok(ImageConfig {
        digest, architecture: r.architecture, os: r.os,
        created: r.created, author: r.author,
        env: r.config.env, cmd: r.config.cmd, entrypoint: r.config.entrypoint,
        working_dir: r.config.working_dir,
        exposed_ports: r.config.exposed_ports, labels: r.config.labels,
        history: r.history.into_iter().map(|h| HistoryEntry { created: h.created, author: h.author, created_by: h.created_by.unwrap_or_default(), empty_layer: h.empty_layer, comment: h.comment }).collect(),
        rootfs: RootFs { kind: r.rootfs.kind, diff_ids: r.rootfs.diff_ids },
        raw_json: String::from_utf8_lossy(raw).to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOCKER_V2_MANIFEST: &str = r#"{
        "schemaVersion": 2,
        "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
        "config": {"mediaType": "application/vnd.docker.container.image.v1+json", "digest": "sha256:abcdef0123456789", "size": 7023},
        "layers": [
            {"mediaType":"application/vnd.docker.image.rootfs.diff.tar.gzip","digest":"sha256:aaa","size":100},
            {"mediaType":"application/vnd.docker.image.rootfs.diff.tar.gzip","digest":"sha256:bbb","size":200}
        ]
    }"#;

    #[test]
    fn parses_docker_v2() {
        let m = parse_manifest(DOCKER_V2_MANIFEST.as_bytes(), "application/vnd.docker.distribution.manifest.v2+json").unwrap();
        assert_eq!(m.kind, ManifestKind::DockerV2);
        assert_eq!(m.layer_count, 2);
        assert_eq!(m.total_size, 300);
        assert!(m.digest.starts_with("sha256:"));
    }

    #[test]
    fn rejects_unsupported_media_type() {
        let err = parse_manifest(b"{}", "application/x-unknown").unwrap_err();
        assert!(matches!(err, ManifestError::UnsupportedMediaType(_)));
    }

    #[test]
    fn parses_image_config() {
        let raw = r#"{"architecture":"amd64","os":"linux","created":"2024-01-01T00:00:00Z","config":{"Env":["FOO=bar"],"Cmd":["sh"]},"history":[{"created":"2024-01-01T00:00:00Z","created_by":"/bin/sh"}],"rootfs":{"type":"layers","diff_ids":["sha256:a","sha256:b"]}}"#;
        let c = parse_image_config(raw.as_bytes()).unwrap();
        assert_eq!(c.architecture, "amd64");
        assert_eq!(c.env, vec!["FOO=bar"]);
        assert_eq!(c.history.len(), 1);
    }
}
