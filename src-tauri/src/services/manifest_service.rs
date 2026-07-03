use crate::error::AppError;
use crate::oci::types::{ArtifactKind, ManifestSummary};
use crate::services::registry_service::RegistryService;
use crate::storage::repo::CacheRepo;
use crate::storage::sqlite::SqliteHandle;
use uuid::Uuid;

pub struct ManifestService<'a> { pub handle: &'a SqliteHandle }

impl<'a> ManifestService<'a> {
    pub fn new(handle: &'a SqliteHandle) -> Self { Self { handle } }

    pub async fn get(&self, connection_id: Uuid, repo: &str, reference: &str) -> Result<(ManifestSummary, Vec<u8>), AppError> {
        let svc = RegistryService::new(self.handle);
        let client = svc.build_client(connection_id)?;
        let (summary, raw) = client.get_manifest(repo, reference).await.map_err(AppError::from)?;
        let mut summary = summary;
        summary.artifact_kind = detect_kind(&summary);
        let cache = CacheRepo::new(self.handle);
        cache.put_manifest(&summary.digest, &summary).map_err(AppError::from)?;
        Ok((summary, raw))
    }

    pub async fn get_config(&self, connection_id: Uuid, repo: &str, digest: &str) -> Result<crate::oci::types::ImageConfig, AppError> {
        let svc = RegistryService::new(self.handle);
        let client = svc.build_client(connection_id)?;
        let bytes = client.get_blob(repo, digest).await.map_err(AppError::from)?;
        crate::oci::parser::parse_image_config(&bytes).map_err(|e| AppError::Registry(crate::registry_error::RegistryError::InvalidManifest(e.to_string())))
    }
}

fn detect_kind(m: &ManifestSummary) -> ArtifactKind {
    if let Some(c) = &m.config_descriptor {
        if c.media_type == "application/vnd.cncf.helm.config.v1+json" { return ArtifactKind::HelmChart; }
    }
    ArtifactKind::Image
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oci::types::ManifestKind;
    #[test]
    fn helm_chart_detected() {
        let m = ManifestSummary {
            digest: "x".into(), media_type: "y".into(), schema_version: 2, kind: ManifestKind::OciManifest,
            total_size: 0, layer_count: 0, config_descriptor: Some(crate::oci::types::Descriptor { media_type: "application/vnd.cncf.helm.config.v1+json".into(), digest: "x".into(), size: 1, urls: None, annotations: None }),
            layer_descriptors: vec![], platforms: vec![], raw_json: "{}".into(), artifact_kind: ArtifactKind::Image,
        };
        assert_eq!(detect_kind(&m), ArtifactKind::HelmChart);
    }
}
