use std::fmt;

use serde::{Deserialize, Serialize};

use crate::domain::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamId(String);

#[allow(dead_code)]
impl TeamId {
    pub fn new(value: String) -> Result<Self, DomainError> {
        if value.is_empty() {
            return Err(DomainError::InvalidInput(
                "team id cannot be empty".to_string(),
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TeamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_string() {
        let result = TeamId::new("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn accepts_non_empty_string() {
        let id = TeamId::new("team-xyz".to_string()).unwrap();
        assert_eq!(id.as_str(), "team-xyz");
    }

    #[test]
    fn equality_by_value() {
        let a = TeamId::new("t-1".to_string()).unwrap();
        let b = TeamId::new("t-1".to_string()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn display_shows_inner_value() {
        let id = TeamId::new("ENG".to_string()).unwrap();
        assert_eq!(id.to_string(), "ENG");
    }
}
