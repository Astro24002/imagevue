use crate::error::AppError;
use crate::oci::auth::{AuthInterceptor, NoopCredentialProvider, TokenCache};
use crate::oci::client::{DistributionClient, ListOptions};
use crate::oci::http::HttpClient;
use crate::storage::repo::{CacheRepo, ConnectionRepo};
use crate::storage::sqlite::SqliteHandle;
use std::sync::Arc;
use uuid::Uuid;

pub struct RegistryService<'a> { pub handle: &'a SqliteHandle }

impl<'a> RegistryService<'a> {
    pub fn new(handle: &'a SqliteHandle) -> Self { Self { handle } }

    pub fn build_client(&self, connection_id: Uuid) -> Result<DistributionClient, AppError> {
        let repo = ConnectionRepo::new(self.handle);
        let conn = repo.get(connection_id).map_err(AppError::from)?;
        let endpoint = if conn.insecure { conn.endpoint.replacen("https://", "http://", 1) } else { conn.endpoint.clone() };
        let http = HttpClient::new();
        let cache = Arc::new(TokenCache::new());
        let creds: Arc<dyn crate::oci::auth::CredentialProvider> = Arc::new(NoopCredentialProvider);
        let auth = Arc::new(AuthInterceptor::new(http.clone(), cache, creds));
        Ok(DistributionClient::new(endpoint, conn.kind, http, auth))
    }

    pub fn ping(&self, connection_id: Uuid) -> Result<(), AppError> {
        let client = self.build_client(connection_id)?;
        let rt = tokio::runtime::Handle::current();
        rt.block_on(client.ping()).map_err(AppError::from)?;
        let conns = ConnectionRepo::new(self.handle);
        conns.mark_connected(connection_id).map_err(AppError::from)?;
        Ok(())
    }

    pub async fn list_repositories_async(&self, connection_id: Uuid, query: &str, limit: u32) -> Result<Vec<crate::oci::types::Repository>, AppError> {
        let client = self.build_client(connection_id)?;
        let page = client.list_repositories(ListOptions { limit, last: None }).await.map_err(AppError::from)?;
        let repos: Vec<_> = page.items.into_iter().filter(|r| r.name.contains(query)).collect();
        let cache = CacheRepo::new(self.handle);
        cache.put_repo_list(connection_id, query, &repos).map_err(AppError::from)?;
        Ok(repos)
    }
}
