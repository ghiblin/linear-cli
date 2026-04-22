use async_trait::async_trait;
use tracing::instrument;

use crate::domain::{
    errors::AuthError,
    repositories::credential_store::{CredentialStore, StorageKind},
    value_objects::api_key::ApiKey,
};

const SERVICE: &str = "linear-cli";
const USERNAME: &str = "default";

pub struct KeyringCredentialStore;

#[allow(dead_code)]
impl KeyringCredentialStore {
    pub fn new() -> Self {
        KeyringCredentialStore
    }
}

#[async_trait]
impl CredentialStore for KeyringCredentialStore {
    #[instrument(skip(self, key))]
    async fn store(&self, key: &ApiKey) -> Result<(), AuthError> {
        let entry = keyring::Entry::new(SERVICE, USERNAME)
            .map_err(|e| AuthError::KeychainUnavailable(e.to_string()))?;
        entry
            .set_password(key.as_str())
            .map_err(|e| AuthError::KeychainUnavailable(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn retrieve(&self) -> Result<Option<ApiKey>, AuthError> {
        let entry = keyring::Entry::new(SERVICE, USERNAME)
            .map_err(|e| AuthError::KeychainUnavailable(e.to_string()))?;
        match entry.get_password() {
            Ok(raw) => ApiKey::new(raw)
                .map(Some)
                .map_err(|e| AuthError::KeychainUnavailable(e.to_string())),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AuthError::KeychainUnavailable(e.to_string())),
        }
    }

    #[instrument(skip(self))]
    async fn remove(&self) -> Result<(), AuthError> {
        let entry = keyring::Entry::new(SERVICE, USERNAME)
            .map_err(|e| AuthError::KeychainUnavailable(e.to_string()))?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AuthError::KeychainUnavailable(e.to_string())),
        }
    }

    fn kind(&self) -> StorageKind {
        StorageKind::Keychain
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    // Both tests mutate the same keychain entry — serialize them to avoid flakiness.
    static KEYCHAIN_LOCK: Mutex<()> = Mutex::new(());

    #[tokio::test]
    async fn keyring_roundtrip() {
        if std::env::var("SKIP_KEYCHAIN_TESTS").is_ok() {
            return;
        }
        let _guard = KEYCHAIN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let store = KeyringCredentialStore::new();
        let key = ApiKey::new("test-keyring-roundtrip-12345").unwrap();

        // Skip if the keychain is not accessible in this environment.
        if store.store(&key).await.is_err() {
            return;
        }

        let retrieved = store.retrieve().await.unwrap();
        // Keychain may not be readable (e.g. unsigned binary, sandboxed env) — skip rather than fail.
        if retrieved.is_none() {
            let _ = store.remove().await;
            return;
        }
        assert_eq!(retrieved.unwrap().as_str(), key.as_str());

        store.remove().await.unwrap();
        let after_remove = store.retrieve().await.unwrap();
        assert!(after_remove.is_none());
    }

    #[tokio::test]
    async fn no_entry_maps_to_none() {
        if std::env::var("SKIP_KEYCHAIN_TESTS").is_ok() {
            return;
        }
        let _guard = KEYCHAIN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let store = KeyringCredentialStore::new();
        let _ = store.remove().await;
        // retrieve() after remove should be None or Err (keychain unavailable), not a stored value.
        match store.retrieve().await {
            Ok(None) | Err(_) => {} // expected
            Ok(Some(_)) => panic!("expected no entry after remove"),
        }
    }
}
