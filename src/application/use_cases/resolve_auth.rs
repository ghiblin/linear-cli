use std::sync::Arc;

use crate::domain::{
    entities::auth_session::{AuthSession, CredentialSource},
    errors::AuthError,
    repositories::{credential_store::CredentialStore, linear_api_client::LinearApiClient},
    value_objects::api_key::ApiKey,
};

#[allow(dead_code)]
pub async fn resolve_auth(
    env_key: Option<ApiKey>,
    stores: Vec<Box<dyn CredentialStore>>,
    client: Arc<dyn LinearApiClient>,
) -> Result<AuthSession, AuthError> {
    if let Some(key) = env_key {
        let login_result = client.validate_api_key(&key).await?;
        return Ok(AuthSession::new(
            key,
            Some(login_result.workspace().clone()),
            CredentialSource::EnvVar,
        ));
    }

    for store in &stores {
        match store.retrieve().await? {
            Some(key) => match client.validate_api_key(&key).await {
                Ok(login_result) => {
                    let source = match store.kind() {
                        crate::domain::repositories::credential_store::StorageKind::Keychain => {
                            CredentialSource::Keychain
                        }
                        crate::domain::repositories::credential_store::StorageKind::File(p) => {
                            CredentialSource::File(p)
                        }
                    };
                    return Ok(AuthSession::new(
                        key,
                        Some(login_result.workspace().clone()),
                        source,
                    ));
                }
                Err(AuthError::InvalidKey) => {
                    store.remove().await.ok();
                    return Err(AuthError::InvalidKey);
                }
                Err(e) => return Err(e),
            },
            None => continue,
        }
    }

    Err(AuthError::NotAuthenticated)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::domain::{
        entities::{login_result::LoginResult, workspace::Workspace},
        repositories::{
            credential_store::MockCredentialStore, linear_api_client::MockLinearApiClient,
        },
        value_objects::api_key::ApiKey,
    };

    fn make_login_result() -> LoginResult {
        let ws = Workspace::new("org-1", "Acme", "acme").unwrap();
        LoginResult::new("user-1", "Alice", ws)
    }

    #[tokio::test]
    async fn env_var_checked_first() {
        let env_key = ApiKey::new("env-key").unwrap();
        let lr = make_login_result();

        let mut mock_client = MockLinearApiClient::new();
        let lr_clone = lr.clone();
        mock_client
            .expect_validate_api_key()
            .returning(move |_| Ok(lr_clone.clone()));

        let result = resolve_auth(Some(env_key), vec![], Arc::new(mock_client)).await;
        assert!(result.is_ok());
        assert!(matches!(result.unwrap().source(), CredentialSource::EnvVar));
    }

    #[tokio::test]
    async fn not_authenticated_when_all_sources_empty() {
        let mock_client = MockLinearApiClient::new();
        let mut mock_store = MockCredentialStore::new();
        mock_store.expect_retrieve().returning(|| Ok(None));

        let result = resolve_auth(None, vec![Box::new(mock_store)], Arc::new(mock_client)).await;
        assert!(matches!(result, Err(AuthError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn revoked_key_calls_remove_then_returns_invalid_key() {
        let stored_key = ApiKey::new("revoked-key").unwrap();

        let mut mock_client = MockLinearApiClient::new();
        mock_client
            .expect_validate_api_key()
            .returning(|_| Err(AuthError::InvalidKey));

        let mut mock_store = MockCredentialStore::new();
        mock_store
            .expect_retrieve()
            .returning(move || Ok(Some(stored_key.clone())));
        mock_store.expect_remove().returning(|| Ok(()));

        let result = resolve_auth(None, vec![Box::new(mock_store)], Arc::new(mock_client)).await;
        assert!(matches!(result, Err(AuthError::InvalidKey)));
    }

    #[tokio::test]
    async fn keychain_source_returns_keychain_session() {
        let stored_key = ApiKey::new("valid-key").unwrap();
        let lr = make_login_result();

        let mut mock_client = MockLinearApiClient::new();
        let lr_clone = lr.clone();
        mock_client
            .expect_validate_api_key()
            .returning(move |_| Ok(lr_clone.clone()));

        let mut mock_store = MockCredentialStore::new();
        mock_store
            .expect_retrieve()
            .returning(move || Ok(Some(stored_key.clone())));
        mock_store
            .expect_kind()
            .returning(|| crate::domain::repositories::credential_store::StorageKind::Keychain);

        let result = resolve_auth(None, vec![Box::new(mock_store)], Arc::new(mock_client)).await;
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap().source(),
            CredentialSource::Keychain
        ));
    }
}
