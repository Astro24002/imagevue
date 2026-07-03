use crate::auth_error::AuthError;
use crate::oci::auth::RegistryKind;
use crate::storage::keyring::KeyringStore;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct OAuthSession {
    pub auth_url: String,
    pub state: String,
    pub code_verifier: String,
}

pub struct AuthService;

impl AuthService {
    pub fn begin_oauth(kind: RegistryKind, _connection_id: Uuid) -> Result<OAuthSession, AuthError> {
        let state = Uuid::new_v4().to_string();
        let code_verifier = generate_verifier();
        let auth_url = match kind {
            RegistryKind::Ghcr => format!(
                "https://github.com/login/oauth/authorize?client_id=imagevue&scope=read:packages&state={state}&code_challenge={code_verifier}&code_challenge_method=S256"),
            RegistryKind::Quay => format!(
                "https://quay.io/oauth/authorize?client_id=imagevue&response_type=code&scope=repo:read&state={state}&code_challenge={code_verifier}&code_challenge_method=S256"),
            RegistryKind::Gcr => format!(
                "https://accounts.google.com/o/oauth2/v2/auth?client_id=imagevue&response_type=code&scope=openid+https://www.googleapis.com/auth/cloud-platform&state={state}&code_challenge={code_verifier}&code_challenge_method=S256"),
            _ => return Err(AuthError::OAuth("registry does not support OAuth flow".into())),
        };
        Ok(OAuthSession { auth_url, state, code_verifier })
    }

    pub fn complete_oauth(
        kind: RegistryKind,
        connection_id: Uuid,
        code: &str,
        state: &str,
        expected_state: &str,
        code_verifier: &str,
    ) -> Result<(), AuthError> {
        if state != expected_state { return Err(AuthError::OAuthStateMismatch); }
        let (token_url, _client_id) = match kind {
            RegistryKind::Ghcr => ("https://github.com/login/oauth/access_token", "imagevue"),
            RegistryKind::Quay => ("https://quay.io/oauth/token", "imagevue"),
            RegistryKind::Gcr => ("https://oauth2.googleapis.com/token", "imagevue"),
            _ => return Err(AuthError::OAuth("registry does not support OAuth flow".into())),
        };
        let rt = tokio::runtime::Handle::current();
        let token_url = token_url.to_string();
        let code = code.to_string();
        let code_verifier = code_verifier.to_string();
        rt.block_on(async {
            let client = reqwest::Client::new();
            let params = [("client_id", "imagevue"), ("code", code.as_str()), ("code_verifier", code_verifier.as_str()), ("grant_type", "authorization_code")];
            let resp = client.post(&token_url).form(&params).send().await.map_err(|e| AuthError::OAuth(e.to_string()))?;
            if !resp.status().is_success() { return Err(AuthError::OAuth(format!("token exchange {}", resp.status().as_u16()))); }
            #[derive(serde::Deserialize)] struct TokenResp { #[serde(default)] access_token: Option<String>, #[serde(default)] refresh_token: Option<String> }
            let tr: TokenResp = resp.json().await.map_err(|e| AuthError::OAuth(e.to_string()))?;
            if let Some(rt) = tr.refresh_token { KeyringStore::set(&format!("oauth:{connection_id}"), &rt)?; }
            else if let Some(at) = tr.access_token { KeyringStore::set(&format!("oauth:{connection_id}"), &at)?; }
            else { return Err(AuthError::OAuth("no token in response".into())); }
            Ok::<_, AuthError>(())
        })
    }
}

fn generate_verifier() -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let bytes: [u8; 32] = rand::random();
    URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dockerhub_oauth_rejected() {
        let s = AuthService::begin_oauth(RegistryKind::DockerHub, Uuid::new_v4());
        assert!(s.is_err());
    }

    #[test]
    fn ghcr_oauth_returns_valid_url() {
        let s = AuthService::begin_oauth(RegistryKind::Ghcr, Uuid::new_v4()).unwrap();
        assert!(s.auth_url.contains("github.com"));
        assert!(!s.state.is_empty());
        assert!(base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&s.code_verifier).is_ok());
    }

    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
}
