use crate::error::AppError;
use crate::services::pull_service::PullService;
use crate::services::registry_service::RegistryService;
use crate::storage::sqlite::SqliteHandle;
use serde::Deserialize;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartPullInput {
    pub connection_id: Uuid,
    pub repository: String,
    pub tag: String,
    pub output_dir: String,
    pub is_chart: bool,
}

#[tauri::command]
pub async fn start_pull(app: AppHandle, handle: State<'_, SqliteHandle>, input: StartPullInput) -> Result<Uuid, AppError> {
    let client = RegistryService::new(&handle).build_client(input.connection_id)?;
    let (tx, mut rx) = mpsc::channel(64);
    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Some(ev) = rx.recv().await {
            let _ = app_clone.emit("pull://progress", &ev);
        }
    });
    let svc = PullService::new();
    let dir = PathBuf::from(input.output_dir);
    if input.is_chart {
        svc.start_chart(client, input.repository, input.tag, dir, tx).await
    } else {
        svc.start_image(client, input.repository, input.tag, dir, tx).await
    }
}
