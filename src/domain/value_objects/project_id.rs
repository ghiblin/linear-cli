use std::fmt;

use crate::domain::errors::DomainError;

#[allow(dead_code)]
fn is_uuid(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 36 {
        return false;
    }
    let dashes = [8, 13, 18, 23];
    for (i, &c) in b.iter().enumerate() {
        if dashes.contains(&i) {
            if c != b'-' {
                return false;
            }
        } else if !c.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

#[allow(dead_code)]
fn is_slug(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '-')
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ProjectId {
    Uuid(String),
    Slug(String),
}

#[allow(dead_code)]
impl ProjectId {
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        if s.is_empty() {
            return Err(DomainError::InvalidInput(
                "project id cannot be empty".to_string(),
            ));
        }
        if is_uuid(s) {
            return Ok(Self::Uuid(s.to_string()));
        }
        if is_slug(s) {
            return Ok(Self::Slug(s.to_string()));
        }
        Err(DomainError::InvalidInput(format!(
            "unrecognised project id '{}'; expected a UUID or a slug (e.g. 'q3-platform')",
            s
        )))
    }

    pub fn as_uuid(&self) -> Option<&str> {
        match self {
            Self::Uuid(s) => Some(s.as_str()),
            Self::Slug(_) => None,
        }
    }

    pub fn as_slug(&self) -> Option<&str> {
        match self {
            Self::Slug(s) => Some(s.as_str()),
            Self::Uuid(_) => None,
        }
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uuid(s) | Self::Slug(s) => write!(f, "{}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_uuid() {
        let id = ProjectId::parse("9cfb482a-81e3-4154-b5b9-2c805e70a02d").unwrap();
        assert!(matches!(id, ProjectId::Uuid(_)));
        assert_eq!(id.as_uuid(), Some("9cfb482a-81e3-4154-b5b9-2c805e70a02d"));
        assert!(id.as_slug().is_none());
    }

    #[test]
    fn parses_slug() {
        let id = ProjectId::parse("q3-platform").unwrap();
        assert!(matches!(id, ProjectId::Slug(_)));
        assert_eq!(id.as_slug(), Some("q3-platform"));
        assert!(id.as_uuid().is_none());
    }

    #[test]
    fn parses_simple_slug() {
        let id = ProjectId::parse("myproject").unwrap();
        assert!(matches!(id, ProjectId::Slug(_)));
    }

    #[test]
    fn rejects_empty() {
        assert!(ProjectId::parse("").is_err());
    }

    #[test]
    fn rejects_invalid_format() {
        assert!(ProjectId::parse("123-not-valid").is_err());
    }

    #[test]
    fn display_returns_inner_value() {
        let id = ProjectId::parse("q3-platform").unwrap();
        assert_eq!(id.to_string(), "q3-platform");
    }

    #[test]
    fn uuid_display() {
        let id = ProjectId::parse("9cfb482a-81e3-4154-b5b9-2c805e70a02d").unwrap();
        assert_eq!(id.to_string(), "9cfb482a-81e3-4154-b5b9-2c805e70a02d");
    }
}
