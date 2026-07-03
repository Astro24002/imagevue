use crate::error::AppError;
use crate::oci::client::DistributionClient;
use crate::tarball::builder::TarballBuilder;
use crate::tarball::repack::rezip;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, Semaphore};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullProgressEvent {
    pub job_id: Uuid,
    pub phase: String,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub current_layer: Option<String>,
    pub layer_index: u32,
    pub layer_count: u32,
    pub speed_bps: u64,
}

pub struct PullService;

impl PullService {
    pub fn new() -> Self { Self }
    pub async fn start_image(&self, client: DistributionClient, repo: String, tag: String, output_dir: PathBuf, tx: mpsc::Sender<PullProgressEvent>) -> Result<Uuid, AppError> {
        let job_id = Uuid::new_v4();
        let tx = tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(PullProgressEvent { job_id, phase: "resolvingManifest".into(), bytes_downloaded: 0, bytes_total: 0, current_layer: None, layer_index: 0, layer_count: 0, speed_bps: 0 }).await;
            let (summary, _raw) = match client.get_manifest(&repo, &tag).await {
                Ok(v) => v, Err(e) => { let _ = tx.send(PullProgressEvent { job_id, phase: format!("failed:{e}"), bytes_downloaded: 0, bytes_total: 0, current_layer: None, layer_index: 0, layer_count: 0, speed_bps: 0 }).await; return; }
            };
            let layer_count = summary.layer_descriptors.len() as u32;
            let bytes_total: u64 = summary.layer_descriptors.iter().map(|d| d.size).sum();
            let _ = tx.send(PullProgressEvent { job_id, phase: "fetchingLayers".into(), bytes_downloaded: 0, bytes_total, current_layer: None, layer_index: 0, layer_count, speed_bps: 0 }).await;

            let sem = Arc::new(Semaphore::new(4));
            let mut downloaded: u64 = 0;
            let mut layer_bytes: Vec<(String, Vec<u8>)> = vec![(String::new(), Vec::new()); layer_count as usize];

            let mut handles = Vec::new();
            for (i, layer) in summary.layer_descriptors.iter().enumerate() {
                let permit = sem.clone().acquire_owned().await.unwrap();
                let client = client.clone();
                let repo = repo.clone();
                let digest = layer.digest.clone();
                let tx = tx.clone();
                let total = bytes_total;
                let started = Instant::now();
                let handle = tokio::spawn(async move {
                    let _permit = permit;
                    let resp = client.stream_blob(&repo, &digest).await.expect("stream");
                    let bytes = resp.bytes().await.expect("bytes").to_vec();
                    let speed = if started.elapsed().as_secs() > 0 { (bytes.len() as u64) / started.elapsed().as_secs() } else { 0 };
                    let _ = tx.send(PullProgressEvent { job_id, phase: "fetchingLayers".into(), bytes_downloaded: bytes.len() as u64, bytes_total: total, current_layer: Some(digest.clone()), layer_index: i as u32, layer_count, speed_bps: speed }).await;
                    (digest, bytes)
                });
                handles.push(handle);
                downloaded += layer.size;
            }
            let _ = downloaded;
            for (i, h) in handles.into_iter().enumerate() {
                if let Ok((d, b)) = h.await { layer_bytes[i] = (d, b); }
            }

            let _ = tx.send(PullProgressEvent { job_id, phase: "assemblingTar".into(), bytes_downloaded: bytes_total, bytes_total, current_layer: None, layer_index: layer_count, layer_count, speed_bps: 0 }).await;

            let filename = format!("{}-{}.tar", repo.replace('/', "_"), tag);
            let path = output_dir.join(&filename);
            let mut b = TarballBuilder::new(&path).expect("builder");
            let config_desc = summary.config_descriptor.clone().expect("config desc");
            let config_bytes = client.get_blob(&repo, &config_desc.digest).await.expect("config blob");
            b.add_config(&config_desc.digest, &config_bytes).expect("config");
            for (d, bytes) in &layer_bytes {
                if bytes.is_empty() { continue; }
                let gz = rezip(bytes).expect("rezip");
                b.add_layer(d, &gz).expect("layer");
            }
            b.set_repos(vec![format!("{repo}:{tag}")]);
            b.finish().expect("finish");

            let _ = tx.send(PullProgressEvent { job_id, phase: "completed".into(), bytes_downloaded: bytes_total, bytes_total, current_layer: None, layer_index: layer_count, layer_count, speed_bps: 0 }).await;
        });
        Ok(job_id)
    }

    pub async fn start_chart(&self, client: DistributionClient, repo: String, version: String, output_dir: PathBuf, tx: mpsc::Sender<PullProgressEvent>) -> Result<Uuid, AppError> {
        let job_id = Uuid::new_v4();
        let tx = tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(PullProgressEvent { job_id, phase: "resolvingManifest".into(), bytes_downloaded: 0, bytes_total: 0, current_layer: None, layer_index: 0, layer_count: 0, speed_bps: 0 }).await;
            let (summary, _raw) = match client.get_manifest(&repo, &version).await {
                Ok(v) => v, Err(e) => { let _ = tx.send(PullProgressEvent { job_id, phase: format!("failed:{e}"), bytes_downloaded: 0, bytes_total: 0, current_layer: None, layer_index: 0, layer_count: 0, speed_bps: 0 }).await; return; }
            };
            let layer = match summary.layer_descriptors.first() { Some(l) => l.clone(), None => { let _ = tx.send(PullProgressEvent { job_id, phase: "failed:no layer".into(), bytes_downloaded: 0, bytes_total: 0, current_layer: None, layer_index: 0, layer_count: 0, speed_bps: 0 }).await; return; } };
            let _ = tx.send(PullProgressEvent { job_id, phase: "fetchingLayers".into(), bytes_downloaded: 0, bytes_total: layer.size, current_layer: Some(layer.digest.clone()), layer_index: 0, layer_count: 1, speed_bps: 0 }).await;
            let bytes = match client.get_blob(&repo, &layer.digest).await {
                Ok(b) => b.to_vec(), Err(e) => { let _ = tx.send(PullProgressEvent { job_id, phase: format!("failed:{e}"), bytes_downloaded: 0, bytes_total: 0, current_layer: None, layer_index: 0, layer_count: 0, speed_bps: 0 }).await; return; }
            };
            let filename = format!("{}-{}.tgz", repo.replace('/', "_"), version);
            let path = output_dir.join(&filename);
            if let Err(e) = std::fs::write(&path, &bytes) { let _ = tx.send(PullProgressEvent { job_id, phase: format!("failed:{e}"), bytes_downloaded: 0, bytes_total: 0, current_layer: None, layer_index: 0, layer_count: 0, speed_bps: 0 }).await; return; }
            let _ = tx.send(PullProgressEvent { job_id, phase: "completed".into(), bytes_downloaded: layer.size, bytes_total: layer.size, current_layer: None, layer_index: 1, layer_count: 1, speed_bps: 0 }).await;
        });
        Ok(job_id)
    }
}

impl Default for PullService { fn default() -> Self { Self::new() } }
