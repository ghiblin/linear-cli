use std::fmt;

use serde::{Deserialize, Serialize};

use crate::domain::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserId(String);

#[allow(dead_code)]
impl UserId {
    pub fn new(value: String) -> Result<Self, DomainError> {
        if value.is_empty() {
            return Err(DomainError::InvalidInput(
                "user id cannot be empty".to_string(),
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_string() {
        assert!(UserId::new("".to_string()).is_err());
    }

    #[test]
    fn accepts_valid_uuid() {
        let id = UserId::new("user-uuid-abc".to_string()).unwrap();
        assert_eq!(id.as_str(), "user-uuid-abc");
    }

    #[test]
    fn display_shows_inner_value() {
        let id = UserId::new("user-123".to_string()).unwrap();
        assert_eq!(id.to_string(), "user-123");
    }
}
