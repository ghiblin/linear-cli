use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("no credentials found; run `linear auth login` or set LINEAR_API_KEY")]
    NotAuthenticated,

    #[error("invalid or expired API key")]
    InvalidKey,

    #[error("API key validation failed: {0}")]
    ValidationFailed(String),

    #[error("could not reach Linear API: {0}")]
    NetworkError(String),

    #[error("system keychain is unavailable: {0}; re-run with --store-file to use file storage")]
    KeychainUnavailable(String),

    #[error("credential file error: {0}")]
    FileError(String),
}

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("not found: {0}")]
    #[allow(dead_code)]
    NotFound(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("not implemented")]
    NotImplemented,
}

#[cfg(test)]
mod auth_error_tests {
    use super::*;

    #[test]
    fn not_authenticated_variant_displays_hint() {
        let err = AuthError::NotAuthenticated;
        assert!(err.to_string().contains("linear auth login"));
    }

    #[test]
    fn invalid_key_variant_is_displayable() {
        let err = AuthError::InvalidKey;
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn validation_failed_includes_message() {
        let err = AuthError::ValidationFailed("bad response".into());
        assert!(err.to_string().contains("bad response"));
    }

    #[test]
    fn network_error_includes_message() {
        let err = AuthError::NetworkError("timeout".into());
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn keychain_unavailable_includes_message() {
        let err = AuthError::KeychainUnavailable("no keyring".into());
        assert!(err.to_string().contains("no keyring"));
    }

    #[test]
    fn file_error_includes_message() {
        let err = AuthError::FileError("permission denied".into());
        assert!(err.to_string().contains("permission denied"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_displays_identifier() {
        let err = DomainError::NotFound("issue-123".to_string());
        assert!(err.to_string().contains("issue-123"));
    }

    #[test]
    fn invalid_input_displays_reason() {
        let err = DomainError::InvalidInput("title cannot be empty".to_string());
        assert!(err.to_string().contains("title cannot be empty"));
    }

    #[test]
    fn not_implemented_is_displayable() {
        let err = DomainError::NotImplemented;
        assert!(!err.to_string().is_empty());
    }
}
