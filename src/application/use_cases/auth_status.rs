use std::sync::Arc;

use crate::domain::{
    entities::auth_session::AuthSession,
    errors::AuthError,
    repositories::{credential_store::CredentialStore, linear_api_client::LinearApiClient},
    value_objects::api_key::ApiKey,
};

use super::resolve_auth::resolve_auth;

#[allow(dead_code)]
pub struct AuthStatusUseCase {
    client: Arc<dyn LinearApiClient>,
}

#[allow(dead_code)]
impl AuthStatusUseCase {
    pub fn new(client: Arc<dyn LinearApiClient>) -> Self {
        AuthStatusUseCase { client }
    }

    pub async fn execute(
        &self,
        env_key: Option<ApiKey>,
        stores: Vec<Box<dyn CredentialStore>>,
    ) -> Result<AuthSession, AuthError> {
        resolve_auth(env_key, stores, self.client.clone()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::domain::{
        entities::workspace::Workspace,
        errors::AuthError,
        repositories::{
            credential_store::MockCredentialStore, linear_api_client::MockLinearApiClient,
        },
        value_objects::api_key::ApiKey,
    };

    fn make_workspace() -> Workspace {
        Workspace::new("org-1", "Acme", "acme").unwrap()
    }

    #[tokio::test]
    async fn authenticated_returns_auth_session() {
        let key = ApiKey::new("valid-key").unwrap();
        let ws = make_workspace();

        let mut mock_client = MockLinearApiClient::new();
        let ws_clone = ws.clone();
        mock_client
            .expect_validate_api_key()
            .returning(move |_| Ok(ws_clone.clone()));

        let mut mock_store = MockCredentialStore::new();
        mock_store
            .expect_retrieve()
            .returning(move || Ok(Some(key.clone())));
        mock_store
            .expect_kind()
            .returning(|| crate::domain::repositories::credential_store::StorageKind::Keychain);

        let use_case = AuthStatusUseCase::new(Arc::new(mock_client));
        let result = use_case.execute(None, vec![Box::new(mock_store)]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn no_credential_returns_not_authenticated() {
        let mock_client = MockLinearApiClient::new();
        let mut mock_store = MockCredentialStore::new();
        mock_store.expect_retrieve().returning(|| Ok(None));

        let use_case = AuthStatusUseCase::new(Arc::new(mock_client));
        let result = use_case.execute(None, vec![Box::new(mock_store)]).await;
        assert!(matches!(result, Err(AuthError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn revoked_key_returns_invalid_key() {
        let key = ApiKey::new("revoked").unwrap();

        let mut mock_client = MockLinearApiClient::new();
        mock_client
            .expect_validate_api_key()
            .returning(|_| Err(AuthError::InvalidKey));

        let mut mock_store = MockCredentialStore::new();
        mock_store
            .expect_retrieve()
            .returning(move || Ok(Some(key.clone())));
        mock_store.expect_remove().returning(|| Ok(()));

        let use_case = AuthStatusUseCase::new(Arc::new(mock_client));
        let result = use_case.execute(None, vec![Box::new(mock_store)]).await;
        assert!(matches!(result, Err(AuthError::InvalidKey)));
    }
}
