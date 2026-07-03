use crate::auth_error::AuthError;
use crate::oci::http::HttpClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RegistryKind {
    DockerHub,
    Ghcr,
    Quay,
    Gcr,
    Generic,
}

impl RegistryKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            RegistryKind::DockerHub => "dockerHub",
            RegistryKind::Ghcr => "ghcr",
            RegistryKind::Quay => "quay",
            RegistryKind::Gcr => "gcr",
            RegistryKind::Generic => "generic",
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Scope {
    Registry,
    Repository,
    Catalog,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ScopeKey {
    pub endpoint: String,
    pub repository: Option<String>,
    pub scope: Scope,
}

#[derive(Debug, Clone)]
pub struct CachedToken {
    pub access_token: String,
    pub expires_at: Instant,
}

pub struct TokenCache {
    inner: Arc<Mutex<HashMap<ScopeKey, CachedToken>>>,
}

impl Default for TokenCache { fn default() -> Self { Self::new() } }

impl TokenCache {
    pub fn new() -> Self { Self { inner: Arc::new(Mutex::new(HashMap::new())) } }
    pub async fn get(&self, key: &ScopeKey) -> Option<String> {
        let g = self.inner.lock().await;
        g.get(key).and_then(|t| if t.expires_at > Instant::now() { Some(t.access_token.clone()) } else { None })
    }
    pub async fn put(&self, key: ScopeKey, token: String, ttl: Duration) {
        let mut g = self.inner.lock().await;
        g.insert(key, CachedToken { access_token: token, expires_at: Instant::now() + ttl });
    }
    pub async fn invalidate(&self, key: &ScopeKey) {
        let mut g = self.inner.lock().await;
        g.remove(key);
    }
}

#[async_trait::async_trait]
pub trait CredentialProvider: Send + Sync {
    async fn basic(&self) -> Option<(String, String)> { None }
    async fn bearer(&self) -> Option<String> { None }
}

pub struct NoopCredentialProvider;

#[async_trait::async_trait]
impl CredentialProvider for NoopCredentialProvider {}

#[allow(dead_code)]
pub struct AuthInterceptor {
    http: HttpClient,
    cache: Arc<TokenCache>,
    creds: Arc<dyn CredentialProvider>,
}

impl AuthInterceptor {
    pub fn new(http: HttpClient, cache: Arc<TokenCache>, creds: Arc<dyn CredentialProvider>) -> Self {
        Self { http, cache, creds }
    }

    pub async fn before_request(
        &self,
        mut req: reqwest::RequestBuilder,
        kind: &RegistryKind,
        repository: Option<String>,
        endpoint: &str,
        scope: Scope,
    ) -> Result<reqwest::RequestBuilder, AuthError> {
        if let Some(b) = self.creds.bearer().await {
            req = req.bearer_auth(b);
            return Ok(req);
        }
        if let Some((u, p)) = self.creds.basic().await {
            req = req.basic_auth(u, Some(p));
            return Ok(req);
        }
        let key = ScopeKey { endpoint: endpoint.to_string(), repository, scope };
        if let Some(tok) = self.cache.get(&key).await {
            req = req.bearer_auth(tok);
        }
        let _ = kind;
        Ok(req)
    }
}
