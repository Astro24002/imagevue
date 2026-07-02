use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),
    #[error("unsupported media type: {0}")]
    UnsupportedMediaType(String),
    #[error("rate limited, retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },
    #[error("server error {status}: {body}")]
    ServerError { status: u16, body: String },
    #[error("network: {0}")]
    Network(String),
}
