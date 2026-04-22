use std::sync::Arc;

use crate::domain::{
    entities::workspace::Workspace,
    errors::AuthError,
    repositories::{credential_store::CredentialStore, linear_api_client::LinearApiClient},
    value_objects::api_key::ApiKey,
};

#[allow(dead_code)]
pub struct LoginUseCase {
    client: Arc<dyn LinearApiClient>,
}

#[allow(dead_code)]
impl LoginUseCase {
    pub fn new(client: Arc<dyn LinearApiClient>) -> Self {
        LoginUseCase { client }
    }

    pub async fn execute(
        &self,
        api_key: ApiKey,
        store: Box<dyn CredentialStore>,
        overwrite: bool,
    ) -> Result<Workspace, AuthError> {
        if !overwrite {
            if let Some(_existing) = store.retrieve().await? {
                return Err(AuthError::NotAuthenticated);
            }
        }

        let workspace = self.client.validate_api_key(&api_key).await?;
        store.store(&api_key).await?;
        Ok(workspace)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate;

    use crate::domain::{
        entities::workspace::Workspace,
        errors::AuthError,
        repositories::{
            credential_store::MockCredentialStore, linear_api_client::MockLinearApiClient,
        },
        value_objects::api_key::ApiKey,
    };

    use super::LoginUseCase;

    fn make_workspace() -> Workspace {
        Workspace::new("org-1", "Acme", "acme").unwrap()
    }

    #[tokio::test]
    async fn valid_key_validates_stores_and_returns_workspace() {
        let key = ApiKey::new("valid-key").unwrap();
        let ws = make_workspace();

        let mut mock_client = MockLinearApiClient::new();
        let ws_clone = ws.clone();
        mock_client
            .expect_validate_api_key()
            .returning(move |_| Ok(ws_clone.clone()));

        let mut mock_store = MockCredentialStore::new();
        mock_store.expect_retrieve().returning(|| Ok(None));
        mock_store
            .expect_store()
            .with(predicate::always())
            .returning(|_| Ok(()));

        let use_case = LoginUseCase::new(Arc::new(mock_client));
        let result = use_case.execute(key, Box::new(mock_store), false).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ws);
    }

    #[tokio::test]
    async fn invalid_key_returns_invalid_key_error() {
        let key = ApiKey::new("bad-key").unwrap();

        let mut mock_client = MockLinearApiClient::new();
        mock_client
            .expect_validate_api_key()
            .returning(|_| Err(AuthError::InvalidKey));

        let mut mock_store = MockCredentialStore::new();
        mock_store.expect_retrieve().returning(|| Ok(None));

        let use_case = LoginUseCase::new(Arc::new(mock_client));
        let result = use_case.execute(key, Box::new(mock_store), false).await;
        assert!(matches!(result, Err(AuthError::InvalidKey)));
    }

    #[tokio::test]
    async fn network_error_returns_network_error() {
        let key = ApiKey::new("any-key").unwrap();

        let mut mock_client = MockLinearApiClient::new();
        mock_client
            .expect_validate_api_key()
            .returning(|_| Err(AuthError::NetworkError("timeout".into())));

        let mut mock_store = MockCredentialStore::new();
        mock_store.expect_retrieve().returning(|| Ok(None));

        let use_case = LoginUseCase::new(Arc::new(mock_client));
        let result = use_case.execute(key, Box::new(mock_store), false).await;
        assert!(matches!(result, Err(AuthError::NetworkError(_))));
    }

    #[tokio::test]
    async fn credential_exists_without_overwrite_returns_not_authenticated() {
        let key = ApiKey::new("new-key").unwrap();
        let existing_key = ApiKey::new("existing-key").unwrap();

        let mock_client = MockLinearApiClient::new();

        let mut mock_store = MockCredentialStore::new();
        mock_store
            .expect_retrieve()
            .returning(move || Ok(Some(existing_key.clone())));

        let use_case = LoginUseCase::new(Arc::new(mock_client));
        let result = use_case.execute(key, Box::new(mock_store), false).await;
        assert!(matches!(result, Err(AuthError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn credential_exists_with_overwrite_validates_and_stores() {
        let key = ApiKey::new("new-key").unwrap();
        let existing_key = ApiKey::new("existing-key").unwrap();
        let ws = make_workspace();

        let mut mock_client = MockLinearApiClient::new();
        let ws_clone = ws.clone();
        mock_client
            .expect_validate_api_key()
            .returning(move |_| Ok(ws_clone.clone()));

        let mut mock_store = MockCredentialStore::new();
        mock_store
            .expect_retrieve()
            .returning(move || Ok(Some(existing_key.clone())));
        mock_store.expect_store().returning(|_| Ok(()));

        let use_case = LoginUseCase::new(Arc::new(mock_client));
        let result = use_case.execute(key, Box::new(mock_store), true).await;
        assert!(result.is_ok());
    }
}
