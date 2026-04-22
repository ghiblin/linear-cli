use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tracing::instrument;

use crate::domain::{
    errors::AuthError,
    repositories::credential_store::{CredentialStore, StorageKind},
    value_objects::api_key::ApiKey,
};

pub struct FileCredentialStore {
    path: PathBuf,
}

#[allow(dead_code)]
impl FileCredentialStore {
    pub fn new() -> Self {
        let path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config/linear-cli/credentials");
        FileCredentialStore { path }
    }

    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        FileCredentialStore { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[async_trait]
impl CredentialStore for FileCredentialStore {
    #[instrument(skip(self, key), fields(path = %self.path.display()))]
    async fn store(&self, key: &ApiKey) -> Result<(), AuthError> {
        eprintln!(
            "Warning: storing credentials in plain-text file: {}",
            self.path.display()
        );

        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| AuthError::FileError(e.to_string()))?;
        }

        std::fs::write(&self.path, key.as_str())
            .map_err(|e| AuthError::FileError(e.to_string()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&self.path, perms)
                .map_err(|e| AuthError::FileError(e.to_string()))?;
        }

        Ok(())
    }

    #[instrument(skip(self), fields(path = %self.path.display()))]
    async fn retrieve(&self) -> Result<Option<ApiKey>, AuthError> {
        match std::fs::read_to_string(&self.path) {
            Ok(raw) => {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    return Ok(None);
                }
                ApiKey::new(trimmed)
                    .map(Some)
                    .map_err(|e| AuthError::FileError(e.to_string()))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(AuthError::FileError(e.to_string())),
        }
    }

    #[instrument(skip(self), fields(path = %self.path.display()))]
    async fn remove(&self) -> Result<(), AuthError> {
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(AuthError::FileError(e.to_string())),
        }
    }

    fn kind(&self) -> StorageKind {
        StorageKind::File(self.path.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn file_store_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("credentials");
        let store = FileCredentialStore::with_path(&path);
        let key = ApiKey::new("file-test-key-12345").unwrap();

        store.store(&key).await.unwrap();
        let retrieved = store.retrieve().await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().as_str(), key.as_str());

        store.remove().await.unwrap();
        let after_remove = store.retrieve().await.unwrap();
        assert!(after_remove.is_none());
    }

    #[tokio::test]
    async fn file_store_missing_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent");
        let store = FileCredentialStore::with_path(path);
        let result = store.retrieve().await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn file_store_remove_missing_is_ok() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent");
        let store = FileCredentialStore::with_path(path);
        assert!(store.remove().await.is_ok());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn file_store_permissions_are_600() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("credentials");
        let store = FileCredentialStore::with_path(&path);
        let key = ApiKey::new("perm-test-key").unwrap();

        store.store(&key).await.unwrap();
        let metadata = std::fs::metadata(&path).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[tokio::test]
    async fn file_store_default_path_is_config_dir() {
        let store = FileCredentialStore::new();
        let expected = dirs::home_dir()
            .unwrap()
            .join(".config/linear-cli/credentials");
        assert_eq!(store.path(), expected);
    }
}
