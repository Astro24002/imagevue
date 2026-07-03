use crate::error::AppError;
use crate::oci::types::{ImageConfig, ManifestSummary};
use crate::services::manifest_service::ManifestService;
use crate::storage::sqlite::SqliteHandle;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn get_manifest(handle: State<'_, SqliteHandle>, connection_id: Uuid, repository: String, reference: String) -> Result<ManifestSummary, AppError> {
    let (m, _raw) = ManifestService::new(&handle).get(connection_id, &repository, &reference).await?;
    Ok(m)
}

#[tauri::command]
pub async fn get_image_config(handle: State<'_, SqliteHandle>, connection_id: Uuid, repository: String, digest: String) -> Result<ImageConfig, AppError> {
    ManifestService::new(&handle).get_config(connection_id, &repository, &digest).await
}
