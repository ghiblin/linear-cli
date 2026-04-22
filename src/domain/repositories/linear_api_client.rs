use async_trait::async_trait;

use crate::domain::{
    entities::workspace::Workspace, errors::AuthError, value_objects::api_key::ApiKey,
};

#[allow(dead_code)]
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait LinearApiClient: Send + Sync {
    async fn validate_api_key(&self, key: &ApiKey) -> Result<Workspace, AuthError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_linear_api_client_compiles() {
        let mut mock = MockLinearApiClient::new();
        mock.expect_validate_api_key()
            .returning(|_| Err(AuthError::InvalidKey));

        let key = ApiKey::new("test-key").unwrap();
        let result = mock.validate_api_key(&key).await;
        assert!(result.is_err());
    }
}
