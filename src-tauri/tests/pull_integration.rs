use imagevue_lib::oci::auth::{AuthInterceptor, NoopCredentialProvider, RegistryKind, TokenCache};
use imagevue_lib::oci::client::DistributionClient;
use imagevue_lib::oci::http::HttpClient;
use imagevue_lib::services::pull_service::PullService;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::mpsc;
use wiremock::{matchers::{method, path}, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn image_pull_with_mock() {
    let dir = tempdir().unwrap();
    let server = MockServer::start().await;
    let manifest = r#"{"schemaVersion":2,"mediaType":"application/vnd.docker.distribution.manifest.v2+json","config":{"mediaType":"application/vnd.docker.container.image.v1+json","digest":"sha256:c","size":2},"layers":[{"mediaType":"application/vnd.docker.image.rootfs.diff.tar.gzip","digest":"sha256:l1","size":4}]}"#;
    Mock::given(method("GET")).and(path("/v2/lib/manifests/v1"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_raw(manifest, "application/vnd.docker.distribution.manifest.v2+json"))
        .mount(&server).await;
    Mock::given(method("GET")).and(path("/v2/lib/blobs/sha256:c"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
        .mount(&server).await;
    let gz = {
        let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        use std::io::Write;
        e.write_all(b"layer1").unwrap();
        e.finish().unwrap()
    };
    Mock::given(method("GET")).and(path("/v2/lib/blobs/sha256:l1"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(gz))
        .mount(&server).await;

    let http = HttpClient::new();
    let cache = Arc::new(TokenCache::new());
    let creds: Arc<dyn imagevue_lib::oci::auth::CredentialProvider> = Arc::new(NoopCredentialProvider);
    let auth = Arc::new(AuthInterceptor::new(http.clone(), cache, creds));
    let client = DistributionClient::new(server.uri(), RegistryKind::Generic, http, auth);
    let svc = PullService::new();
    let (tx, mut rx) = mpsc::channel(16);
    let job = svc.start_image(client, "lib".into(), "v1".into(), dir.path().to_path_buf(), tx).await.unwrap();
    let mut completed = false;
    for _ in 0..30 {
        if let Some(ev) = rx.recv().await {
            if ev.phase == "completed" && ev.job_id == job { completed = true; break; }
        } else { break; }
    }
    assert!(completed);
    assert!(dir.path().join("lib-v1.tar").exists());
}
