use std::path::PathBuf;

use async_trait::async_trait;

use crate::domain::{errors::AuthError, value_objects::api_key::ApiKey};

#[allow(dead_code)]
pub enum StorageKind {
    Keychain,
    File(PathBuf),
}

#[allow(dead_code)]
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CredentialStore: Send + Sync {
    async fn store(&self, key: &ApiKey) -> Result<(), AuthError>;
    async fn retrieve(&self) -> Result<Option<ApiKey>, AuthError>;
    async fn remove(&self) -> Result<(), AuthError>;
    fn kind(&self) -> StorageKind;
}
