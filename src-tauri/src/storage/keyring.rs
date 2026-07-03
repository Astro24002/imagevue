use crate::auth_error::AuthError;
use keyring::Entry;

const SERVICE: &str = "imagevue";

pub struct KeyringStore;

impl KeyringStore {
    pub fn set(account: &str, secret: &str) -> Result<(), AuthError> {
        let entry = Entry::new(SERVICE, account).map_err(|e| AuthError::Keyring(e.to_string()))?;
        entry.set_password(secret).map_err(|e| AuthError::Keyring(e.to_string()))?;
        Ok(())
    }

    pub fn get(account: &str) -> Result<Option<String>, AuthError> {
        let entry = Entry::new(SERVICE, account).map_err(|e| AuthError::Keyring(e.to_string()))?;
        match entry.get_password() {
            Ok(p) => Ok(Some(p)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AuthError::Keyring(e.to_string())),
        }
    }

    pub fn delete(account: &str) -> Result<(), AuthError> {
        let entry = Entry::new(SERVICE, account).map_err(|e| AuthError::Keyring(e.to_string()))?;
        match entry.delete_credential() {
            Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AuthError::Keyring(e.to_string())),
        }
    }
}
