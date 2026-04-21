use thiserror::Error;

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
