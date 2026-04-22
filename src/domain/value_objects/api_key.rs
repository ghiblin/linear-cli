use std::fmt;

use crate::domain::errors::DomainError;

pub struct ApiKey(String);

#[allow(dead_code)]
impl ApiKey {
    pub fn new(raw: impl Into<String>) -> Result<Self, DomainError> {
        let s = raw.into();
        if s.is_empty() {
            return Err(DomainError::InvalidInput("api key cannot be empty".into()));
        }
        Ok(ApiKey(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApiKey([REDACTED])")
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl Clone for ApiKey {
    fn clone(&self) -> Self {
        ApiKey(self.0.clone())
    }
}

impl PartialEq for ApiKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ApiKey {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_empty_string() {
        let result = ApiKey::new("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("api key cannot be empty"));
    }

    #[test]
    fn new_accepts_non_empty_string() {
        let key = ApiKey::new("lin_api_abc123").unwrap();
        assert_eq!(key.as_str(), "lin_api_abc123");
    }

    #[test]
    fn debug_redacts_key_value() {
        let key = ApiKey::new("secret").unwrap();
        let debug_str = format!("{:?}", key);
        assert!(!debug_str.contains("secret"));
        assert!(debug_str.contains("[REDACTED]"));
    }

    #[test]
    fn display_redacts_key_value() {
        let key = ApiKey::new("secret").unwrap();
        let display_str = format!("{}", key);
        assert!(!display_str.contains("secret"));
        assert_eq!(display_str, "[REDACTED]");
    }

    #[test]
    fn as_str_returns_raw_value() {
        let key = ApiKey::new("raw_key_value").unwrap();
        assert_eq!(key.as_str(), "raw_key_value");
    }

    #[test]
    fn clone_produces_equal_key() {
        let key = ApiKey::new("my_key").unwrap();
        let cloned = key.clone();
        assert_eq!(key, cloned);
    }
}
