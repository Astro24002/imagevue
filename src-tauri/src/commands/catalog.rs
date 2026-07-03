use crate::error::AppError;
use crate::oci::types::Repository;
use crate::services::registry_service::RegistryService;
use crate::storage::sqlite::SqliteHandle;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn list_repositories(handle: State<'_, SqliteHandle>, connection_id: Uuid, query: String, limit: u32) -> Result<Vec<Repository>, AppError> {
    RegistryService::new(&handle).list_repositories_async(connection_id, &query, limit).await
}
