use crate::oci::auth::{AuthInterceptor, RegistryKind, Scope};
use crate::oci::http::{HttpClient, HttpError};
use crate::oci::parser::parse_manifest;
use crate::oci::types::*;
use crate::registry_error::RegistryError;
use bytes::Bytes;
use std::sync::Arc;

#[derive(Clone)]
pub struct DistributionClient {
    pub endpoint: String,
    pub kind: RegistryKind,
    http: HttpClient,
    auth: Arc<AuthInterceptor>,
}

pub struct ListOptions {
    pub limit: u32,
    pub last: Option<String>,
}

impl Default for ListOptions {
    fn default() -> Self { Self { limit: 50, last: None } }
}

pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

impl DistributionClient {
    pub fn new(
        endpoint: impl Into<String>,
        kind: RegistryKind,
        http: HttpClient,
        auth: Arc<AuthInterceptor>,
    ) -> Self {
        Self { endpoint: endpoint.into().trim_end_matches('/').to_string(), kind, http, auth }
    }

    pub fn http(&self) -> &HttpClient { &self.http }
    pub fn auth(&self) -> &AuthInterceptor { &self.auth }

    fn url(&self, path: &str) -> String {
        format!("{}/v2{}", self.endpoint, path)
    }

    pub async fn ping(&self) -> Result<(), RegistryError> {
        let req = self.http.inner().get(self.url("/"));
        let req = self.auth.before_request(req, &self.kind, None, &self.endpoint, Scope::Registry).await.map_err(RegistryError::from)?;
        let resp = self.http.execute(req).await.map_err(RegistryError::from)?;
        if resp.status().is_success() { Ok(()) } else { Err(RegistryError::ServerError { status: resp.status().as_u16(), body: "ping failed".into() }) }
    }

    pub async fn list_repositories(&self, opts: ListOptions) -> Result<Page<Repository>, RegistryError> {
        let mut url = format!("{}/_catalog?n={}", self.url(""), opts.limit);
        if let Some(last) = opts.last.as_ref() { url.push_str(&format!("&last={}", last)); }
        let req = self.http.inner().get(&url);
        let req = self.auth.before_request(req, &self.kind, None, &self.endpoint, Scope::Catalog).await.map_err(RegistryError::from)?;
        let resp = self.http.execute(req).await.map_err(RegistryError::from)?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(Page { items: vec![], next_cursor: None });
        }
        if !resp.status().is_success() {
            return Err(RegistryError::ServerError { status: resp.status().as_u16(), body: resp.text().await.unwrap_or_default() });
        }
        #[derive(serde::Deserialize)] struct Resp { repositories: Vec<String> }
        let parsed: Resp = resp.json().await.map_err(|e| RegistryError::Network(e.to_string()))?;
        Ok(Page { items: parsed.repositories.into_iter().map(|n| Repository { name: n }).collect(), next_cursor: None })
    }

    pub async fn list_tags(&self, repo: &str, opts: ListOptions) -> Result<Page<Tag>, RegistryError> {
        let mut url = format!("{}/{}/tags/list?n={}", self.url(""), repo, opts.limit);
        if let Some(last) = opts.last.as_ref() { url.push_str(&format!("&last={}", last)); }
        let req = self.http.inner().get(&url);
        let req = self.auth.before_request(req, &self.kind, Some(repo.to_string()), &self.endpoint, Scope::Repository).await.map_err(RegistryError::from)?;
        let resp = self.http.execute(req).await.map_err(RegistryError::from)?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(Page { items: vec![], next_cursor: None });
        }
        if !resp.status().is_success() {
            return Err(RegistryError::ServerError { status: resp.status().as_u16(), body: resp.text().await.unwrap_or_default() });
        }
        #[derive(serde::Deserialize)] struct Resp { tags: Option<Vec<String>> }
        let parsed: Resp = resp.json().await.map_err(|e| RegistryError::Network(e.to_string()))?;
        Ok(Page {
            items: parsed.tags.unwrap_or_default().into_iter().map(|n| Tag {
                name: n, digest: String::new(), size: 0, updated_at: None,
                os: None, architecture: None, artifact_kind: ArtifactKind::Image,
            }).collect(),
            next_cursor: None,
        })
    }

    pub async fn get_manifest(&self, repo: &str, reference: &str) -> Result<(ManifestSummary, Vec<u8>), RegistryError> {
        let url = self.url(&format!("/{}/manifests/{}", repo, reference));
        let accept = "application/vnd.docker.distribution.manifest.v2+json, application/vnd.docker.distribution.manifest.v1+json, application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.oci.image.manifest.v1+json, application/vnd.oci.image.index.v1+json";
        let req = self.http.inner().get(&url).header("Accept", accept);
        let req = self.auth.before_request(req, &self.kind, Some(repo.to_string()), &self.endpoint, Scope::Repository).await.map_err(RegistryError::from)?;
        let resp = self.http.execute(req).await.map_err(RegistryError::from)?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND { return Err(RegistryError::NotFound(format!("manifest {repo}:{reference}"))); }
        if !resp.status().is_success() { return Err(RegistryError::ServerError { status: resp.status().as_u16(), body: resp.text().await.unwrap_or_default() }); }
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
        let raw = resp.bytes().await.map_err(|e| RegistryError::Network(e.to_string()))?;
        let summary = parse_manifest(&raw, &content_type).map_err(|e| RegistryError::InvalidManifest(e.to_string()))?;
        Ok((summary, raw.to_vec()))
    }

    pub async fn get_blob(&self, repo: &str, digest: &str) -> Result<Bytes, RegistryError> {
        let url = self.url(&format!("/{}/blobs/{}", repo, digest));
        let req = self.http.inner().get(&url);
        let req = self.auth.before_request(req, &self.kind, Some(repo.to_string()), &self.endpoint, Scope::Repository).await.map_err(RegistryError::from)?;
        let resp = self.http.execute(req).await.map_err(RegistryError::from)?;
        if !resp.status().is_success() { return Err(RegistryError::ServerError { status: resp.status().as_u16(), body: resp.text().await.unwrap_or_default() }); }
        Ok(resp.bytes().await.map_err(|e| RegistryError::Network(e.to_string()))?)
    }

    pub async fn blob_exists(&self, repo: &str, digest: &str) -> Result<bool, RegistryError> {
        let url = self.url(&format!("/{}/blobs/{}", repo, digest));
        let req = self.http.inner().head(&url);
        let req = self.auth.before_request(req, &self.kind, Some(repo.to_string()), &self.endpoint, Scope::Repository).await.map_err(RegistryError::from)?;
        let resp = self.http.execute(req).await.map_err(RegistryError::from)?;
        Ok(resp.status().is_success())
    }

    pub async fn stream_blob(&self, repo: &str, digest: &str) -> Result<reqwest::Response, RegistryError> {
        let url = self.url(&format!("/{}/blobs/{}", repo, digest));
        let req = self.http.inner().get(&url);
        let req = self.auth.before_request(req, &self.kind, Some(repo.to_string()), &self.endpoint, Scope::Repository).await.map_err(RegistryError::from)?;
        let resp = self.http.execute(req).await.map_err(RegistryError::from)?;
        if !resp.status().is_success() { return Err(RegistryError::ServerError { status: resp.status().as_u16(), body: "stream failed".into() }); }
        Ok(resp)
    }
}

impl From<HttpError> for RegistryError {
    fn from(e: HttpError) -> Self {
        match e {
            HttpError::Network(m) | HttpError::InvalidUrl(m) => RegistryError::Network(m),
            HttpError::Timeout => RegistryError::Network("timeout".into()),
            HttpError::Status { status, body, retry_after_ms } => {
                if status == 429 { RegistryError::RateLimited { retry_after_ms: retry_after_ms.unwrap_or(0) } }
                else if status == 404 { RegistryError::NotFound(body) }
                else { RegistryError::ServerError { status, body } }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oci::auth::{AuthInterceptor, NoopCredentialProvider, RegistryKind, TokenCache};
    use wiremock::{matchers::{method, path}, Mock, MockServer, ResponseTemplate};

    fn make_client(base_url: String) -> DistributionClient {
        let http = HttpClient::new();
        let cache = Arc::new(TokenCache::new());
        let creds: Arc<dyn crate::oci::auth::CredentialProvider> = Arc::new(NoopCredentialProvider);
        let auth = Arc::new(AuthInterceptor::new(http.clone(), cache, creds));
        DistributionClient::new(base_url, RegistryKind::Generic, http, auth)
    }

    #[tokio::test]
    async fn ping_succeeds_on_200() {
        let server = MockServer::start().await;
        Mock::given(method("GET")).and(path("/v2/"))
            .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
            .mount(&server).await;
        let c = make_client(server.uri());
        c.ping().await.unwrap();
    }

    #[tokio::test]
    async fn list_repositories_parses() {
        let server = MockServer::start().await;
        Mock::given(method("GET")).and(path("/v2/_catalog"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"repositories":["a","b","c"]}"#))
            .mount(&server).await;
        let c = make_client(server.uri());
        let page = c.list_repositories(ListOptions { limit: 50, last: None }).await.unwrap();
        assert_eq!(page.items.len(), 3);
    }

    #[tokio::test]
    async fn list_tags_parses() {
        let server = MockServer::start().await;
        Mock::given(method("GET")).and(path("/v2/lib/tags/list"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"name":"lib","tags":["v1","v2"]}"#))
            .mount(&server).await;
        let c = make_client(server.uri());
        let page = c.list_tags("lib", ListOptions::default()).await.unwrap();
        assert_eq!(page.items.len(), 2);
    }

    #[tokio::test]
    async fn get_manifest_parses() {
        let server = MockServer::start().await;
        let manifest = r#"{"schemaVersion":2,"mediaType":"application/vnd.docker.distribution.manifest.v2+json","config":{"mediaType":"application/vnd.docker.container.image.v1+json","digest":"sha256:a","size":1},"layers":[]}"#;
        Mock::given(method("GET")).and(path("/v2/lib/manifests/v1"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_raw(manifest, "application/vnd.docker.distribution.manifest.v2+json"))
            .mount(&server).await;
        let c = make_client(server.uri());
        let (m, raw) = c.get_manifest("lib", "v1").await.unwrap();
        assert_eq!(m.kind, ManifestKind::DockerV2);
        assert_eq!(raw.len(), manifest.len());
    }
}
