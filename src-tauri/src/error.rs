use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Registry(#[from] crate::registry_error::RegistryError),
    #[error(transparent)]
    Auth(#[from] crate::auth_error::AuthError),
    #[error(transparent)]
    Storage(#[from] crate::storage_error::StorageError),
    #[error(transparent)]
    Pull(#[from] crate::pull_error::PullError),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("AppError", 2)?;
        let (kind, message) = match self {
            AppError::Registry(e) => ("registry", e.to_string()),
            AppError::Auth(e) => ("auth", e.to_string()),
            AppError::Storage(e) => ("storage", e.to_string()),
            AppError::Pull(e) => ("pull", e.to_string()),
            AppError::InvalidInput(m) => ("invalidInput", m.clone()),
            AppError::Internal(m) => ("internal", m.clone()),
        };
        st.serialize_field("kind", kind)?;
        st.serialize_field("message", &message)?;
        st.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_invalid_input() {
        let e = AppError::InvalidInput("bad name".into());
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["kind"], "invalidInput");
        assert_eq!(v["message"], "bad name");
    }

    #[test]
    fn serializes_internal() {
        let e = AppError::Internal("oops".into());
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["kind"], "internal");
    }
}
