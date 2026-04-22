use crate::domain::{
    errors::AuthError,
    repositories::credential_store::{CredentialStore, StorageKind},
};

#[allow(dead_code)]
pub struct LogoutUseCase;

#[allow(dead_code)]
impl LogoutUseCase {
    pub fn new() -> Self {
        LogoutUseCase
    }

    pub async fn execute(
        &self,
        stores: Vec<Box<dyn CredentialStore>>,
        dry_run: bool,
    ) -> Result<Vec<StorageKind>, AuthError> {
        let mut found = Vec::new();

        for store in &stores {
            if store.retrieve().await?.is_some() {
                found.push(store.kind());
            }
        }

        if dry_run {
            return Ok(found);
        }

        for store in &stores {
            if store.retrieve().await?.is_some() {
                store.remove().await?;
            }
        }

        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        repositories::credential_store::{MockCredentialStore, StorageKind},
        value_objects::api_key::ApiKey,
    };

    #[tokio::test]
    async fn credential_present_removes_from_all_stores() {
        let key = ApiKey::new("existing-key").unwrap();

        let mut mock_store = MockCredentialStore::new();
        let key_clone = key.clone();
        mock_store
            .expect_retrieve()
            .times(2)
            .returning(move || Ok(Some(key_clone.clone())));
        mock_store.expect_kind().returning(|| StorageKind::Keychain);
        mock_store.expect_remove().returning(|| Ok(()));

        let use_case = LogoutUseCase::new();
        let result = use_case.execute(vec![Box::new(mock_store)], false).await;
        assert!(result.is_ok());
        let removed = result.unwrap();
        assert!(!removed.is_empty());
    }

    #[tokio::test]
    async fn no_credential_returns_empty() {
        let mut mock_store = MockCredentialStore::new();
        mock_store.expect_retrieve().returning(|| Ok(None));

        let use_case = LogoutUseCase::new();
        let result = use_case.execute(vec![Box::new(mock_store)], false).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn dry_run_returns_what_would_be_removed_without_calling_remove() {
        let key = ApiKey::new("existing-key").unwrap();

        let mut mock_store = MockCredentialStore::new();
        mock_store
            .expect_retrieve()
            .returning(move || Ok(Some(key.clone())));
        mock_store.expect_kind().returning(|| StorageKind::Keychain);

        let use_case = LogoutUseCase::new();
        let result = use_case.execute(vec![Box::new(mock_store)], true).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }
}
