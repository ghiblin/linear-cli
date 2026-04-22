#![allow(dead_code)]

use std::path::PathBuf;

use crate::domain::{entities::workspace::Workspace, value_objects::api_key::ApiKey};

#[derive(Debug, Clone)]
pub enum CredentialSource {
    EnvVar,
    Keychain,
    File(PathBuf),
}

#[derive(Debug, Clone)]
pub struct AuthSession {
    api_key: ApiKey,
    workspace: Option<Workspace>,
    source: CredentialSource,
}

impl AuthSession {
    pub fn new(api_key: ApiKey, workspace: Option<Workspace>, source: CredentialSource) -> Self {
        AuthSession {
            api_key,
            workspace,
            source,
        }
    }

    pub fn api_key(&self) -> &ApiKey {
        &self.api_key
    }

    pub fn workspace(&self) -> Option<&Workspace> {
        self.workspace.as_ref()
    }

    pub fn source(&self) -> &CredentialSource {
        &self.source
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::api_key::ApiKey;

    #[test]
    fn new_creates_auth_session_with_correct_fields() {
        let key = ApiKey::new("my-key").unwrap();
        let session = AuthSession::new(key.clone(), None, CredentialSource::Keychain);
        assert_eq!(session.api_key(), &key);
        assert!(session.workspace().is_none());
        assert!(matches!(session.source(), CredentialSource::Keychain));
    }

    #[test]
    fn credential_source_env_var_variant() {
        let key = ApiKey::new("env-key").unwrap();
        let session = AuthSession::new(key, None, CredentialSource::EnvVar);
        assert!(matches!(session.source(), CredentialSource::EnvVar));
    }

    #[test]
    fn credential_source_file_variant() {
        use std::path::PathBuf;
        let key = ApiKey::new("file-key").unwrap();
        let path = PathBuf::from("/home/user/.config/linear-cli/credentials");
        let session = AuthSession::new(key, None, CredentialSource::File(path.clone()));
        match session.source() {
            CredentialSource::File(p) => assert_eq!(p, &path),
            _ => panic!("expected File variant"),
        }
    }
}
