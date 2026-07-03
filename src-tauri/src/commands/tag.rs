use crate::error::AppError;
use crate::oci::client::ListOptions;
use crate::oci::types::Tag;
use crate::services::registry_service::RegistryService;
use crate::storage::repo::CacheRepo;
use crate::storage::sqlite::SqliteHandle;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn list_tags(handle: State<'_, SqliteHandle>, connection_id: Uuid, repository: String) -> Result<Vec<Tag>, AppError> {
    let svc = RegistryService::new(&handle);
    let client = svc.build_client(connection_id)?;
    let page = client.list_tags(&repository, ListOptions { limit: 200, last: None }).await.map_err(AppError::from)?;
    CacheRepo::new(&handle).put_tag_list(connection_id, &repository, &page.items).map_err(AppError::from)?;
    Ok(page.items)
}
