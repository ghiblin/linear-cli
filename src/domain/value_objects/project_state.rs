use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::domain::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectState {
    Planned,
    Started,
    Paused,
    Completed,
    Cancelled,
}

impl ProjectState {
    pub fn valid_values() -> &'static [&'static str] {
        &["planned", "started", "paused", "completed", "cancelled"]
    }
}

impl FromStr for ProjectState {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "planned" => Ok(Self::Planned),
            "started" => Ok(Self::Started),
            "paused" => Ok(Self::Paused),
            "completed" => Ok(Self::Completed),
            "cancelled" | "canceled" => Ok(Self::Cancelled),
            _ => Err(DomainError::InvalidInput(format!(
                "invalid state '{}'; valid values: {}",
                s,
                Self::valid_values().join(", ")
            ))),
        }
    }
}

impl fmt::Display for ProjectState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Planned => "planned",
            Self::Started => "started",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_states() {
        assert_eq!("planned".parse::<ProjectState>().unwrap(), ProjectState::Planned);
        assert_eq!("started".parse::<ProjectState>().unwrap(), ProjectState::Started);
        assert_eq!("paused".parse::<ProjectState>().unwrap(), ProjectState::Paused);
        assert_eq!("completed".parse::<ProjectState>().unwrap(), ProjectState::Completed);
        assert_eq!("cancelled".parse::<ProjectState>().unwrap(), ProjectState::Cancelled);
        assert_eq!("canceled".parse::<ProjectState>().unwrap(), ProjectState::Cancelled);
    }

    #[test]
    fn parse_is_case_insensitive() {
        assert_eq!("PLANNED".parse::<ProjectState>().unwrap(), ProjectState::Planned);
        assert_eq!("Started".parse::<ProjectState>().unwrap(), ProjectState::Started);
    }

    #[test]
    fn parse_invalid_returns_error_with_valid_values() {
        let err = "bogus".parse::<ProjectState>().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("bogus"));
        assert!(msg.contains("planned"));
    }

    #[test]
    fn display_is_lowercase() {
        assert_eq!(ProjectState::Planned.to_string(), "planned");
        assert_eq!(ProjectState::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn roundtrip_serialize_deserialize() {
        let state = ProjectState::Started;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"started\"");
        let parsed: ProjectState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, state);
    }
}
