use std::fmt;

use serde::{Deserialize, Serialize};

use crate::domain::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelId(String);

impl LabelId {
    pub fn new(s: String) -> Result<Self, DomainError> {
        if s.is_empty() {
            return Err(DomainError::InvalidInput(
                "label id cannot be empty".to_string(),
            ));
        }
        Ok(Self(s))
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LabelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
