use thiserror::Error;

use crate::domain::errors::{AuthError, DomainError};

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::errors::DomainError;

    #[test]
    fn domain_variant_wraps_domain_error() {
        let domain_err = DomainError::NotFound("team-1".to_string());
        let app_err = ApplicationError::Domain(domain_err);
        assert!(app_err.to_string().contains("team-1"));
    }
}
