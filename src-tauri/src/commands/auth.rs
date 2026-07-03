use crate::error::AppError;
use crate::oci::auth::RegistryKind;
use crate::services::auth_service::{AuthService, OAuthSession};
use serde::Deserialize;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeginOAuthInput {
    pub kind: RegistryKind,
    pub connection_id: Uuid,
}

#[tauri::command]
pub async fn begin_oauth(app: AppHandle, input: BeginOAuthInput) -> Result<OAuthSession, AppError> {
    let session = AuthService::begin_oauth(input.kind, input.connection_id)?;
    let _ = app.emit("auth://oauth-session", &session);
    Ok(session)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteOAuthInput {
    pub kind: RegistryKind,
    pub connection_id: Uuid,
    pub code: String,
    pub state: String,
    pub expected_state: String,
    pub code_verifier: String,
}

#[tauri::command]
pub async fn complete_oauth(input: CompleteOAuthInput) -> Result<(), AppError> {
    AuthService::complete_oauth(input.kind, input.connection_id, &input.code, &input.state, &input.expected_state, &input.code_verifier)?;
    Ok(())
}
