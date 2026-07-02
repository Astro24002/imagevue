use thiserror::Error;

#[derive(Debug, Error)]
pub enum PullError {
    #[error("cancelled")]
    Cancelled,
    #[error("digest mismatch: expected {expected}, actual {actual}")]
    DigestMismatch { expected: String, actual: String },
    #[error("disk full at {path}, need {needed_bytes}, have {available_bytes}")]
    DiskFull { path: String, needed_bytes: u64, available_bytes: u64 },
    #[error("decompress: {0}")]
    Decompress(String),
    #[error("regzip: {0}")]
    Regzip(String),
    #[error("write tar: {0}")]
    WriteTar(String),
    #[error(transparent)]
    Registry(#[from] crate::registry_error::RegistryError),
}
