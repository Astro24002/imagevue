use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("unauthorized")]
    Unauthorized { kind: String, connection_id: Uuid },
    #[error("oauth state mismatch")]
    OAuthStateMismatch,
    #[error("oauth callback missing code")]
    OAuthMissingCode,
    #[error("oauth: {0}")]
    OAuth(String),
    #[error("keyring: {0}")]
    Keyring(String),
}
