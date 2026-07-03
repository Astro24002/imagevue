use crate::error::AppError;
use crate::oci::auth::RegistryKind;
use crate::services::registry_service::RegistryService;
use crate::storage::repo::{ConnectionRepo, NewConnection, RegistryConnection};
use crate::storage::sqlite::SqliteHandle;
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionInput {
    pub name: String,
    pub kind: RegistryKind,
    pub endpoint: String,
    pub insecure: bool,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[tauri::command]
pub async fn list_connections(handle: State<'_, SqliteHandle>) -> Result<Vec<RegistryConnection>, AppError> {
    ConnectionRepo::new(&handle).list().map_err(AppError::from)
}

#[tauri::command]
pub async fn get_connection(handle: State<'_, SqliteHandle>, id: Uuid) -> Result<RegistryConnection, AppError> {
    ConnectionRepo::new(&handle).get(id).map_err(AppError::from)
}

#[tauri::command]
pub async fn create_connection(handle: State<'_, SqliteHandle>, input: CreateConnectionInput) -> Result<RegistryConnection, AppError> {
    let cred_ref = match (&input.username, &input.password) {
        (Some(_), Some(p)) => {
            let key = format!("cred:{}", Uuid::new_v4());
            crate::storage::keyring::KeyringStore::set(&key, p)?;
            Some(key)
        }
        _ => None,
    };
    ConnectionRepo::new(&handle).create(NewConnection { name: input.name, kind: input.kind, endpoint: input.endpoint, insecure: input.insecure, credential_ref: cred_ref }).map_err(AppError::from)
}

#[tauri::command]
pub async fn delete_connection(handle: State<'_, SqliteHandle>, id: Uuid) -> Result<(), AppError> {
    if let Ok(c) = ConnectionRepo::new(&handle).get(id) {
        if let Some(k) = c.credential_ref { let _ = crate::storage::keyring::KeyringStore::delete(&k); }
    }
    ConnectionRepo::new(&handle).delete(id).map_err(AppError::from)
}

#[tauri::command]
pub async fn test_connection(handle: State<'_, SqliteHandle>, id: Uuid) -> Result<(), AppError> {
    RegistryService::new(&handle).ping(id)
}
